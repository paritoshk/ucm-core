//! ContextQA API server — Axum REST endpoints for the context graph.
//!
//! Routes:
//! - GET  /health             — health check
//! - GET  /graph/stats        — graph statistics
//! - GET  /graph              — full graph JSON
//! - GET  /graph/entities     — list all entities
//! - GET  /graph/edges        — list all edges with endpoints
//! - POST /ingest/code        — ingest source code
//! - POST /ingest/diff        — ingest a before/after diff
//! - POST /ingest/ticket      — ingest Jira tickets
//! - POST /impact             — analyze impact of changes
//! - POST /intent             — generate test intent from impact report

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::{Arc, Mutex};
use tower_http::cors::CorsLayer;
use uuid::Uuid;

use context_core::edge::{ContextEdge, RelationType};
use context_core::entity::*;
use context_core::graph::ContextGraph;
use context_events::projection::GraphProjection;
use context_events::store::EventStore;
use context_ingest::{code_parser, diff_parser, jira_adapter};
use context_observe::trace::{trace_impact_analysis, TraceStore};
use context_reason::impact::analyze_impact;
use context_reason::intent::generate_test_intent;

/// Shared application state.
struct AppState {
    graph: Mutex<ContextGraph>,
    event_store: Mutex<EventStore>,
    trace_store: Mutex<TraceStore>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // Seed the demo graph on startup
    let mut graph = ContextGraph::new();
    seed_demo_graph(&mut graph);
    tracing::info!("Seeded demo graph: {:?}", graph.stats());

    let state = Arc::new(AppState {
        graph: Mutex::new(graph),
        event_store: Mutex::new(EventStore::new()),
        trace_store: Mutex::new(TraceStore::new()),
    });

    let app = Router::new()
        .route("/health", get(health))
        .route("/graph/stats", get(graph_stats))
        .route("/graph", get(graph_json))
        .route("/graph/entities", get(graph_entities))
        .route("/graph/edges", get(graph_edges))
        .route("/ingest/code", post(ingest_code))
        .route("/ingest/diff", post(ingest_diff))
        .route("/ingest/ticket", post(ingest_ticket))
        .route("/impact", post(impact_analysis))
        .route("/intent", post(test_intent))
        .layer(CorsLayer::permissive())
        .with_state(state);

    // PORT env var for Railway compatibility (defaults to 3001)
    let port = env::var("PORT").unwrap_or_else(|_| "3001".to_string());
    let bind_addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&bind_addr).await.unwrap();
    tracing::info!("ContextQA API server listening on http://{bind_addr}");
    axum::serve(listener, app).await.unwrap();
}

// ─── Demo Graph Seeder ─────────────────────────────────────────────

/// Seeds the graph with the auth→middleware→payment demo scenario.
/// Includes 7 entities and 5 edges matching the dashboard's demo data.
fn seed_demo_graph(graph: &mut ContextGraph) {
    // Entities
    let auth_svc = ContextEntity::new(
        EntityId::local("src/auth/service.ts", "validateToken"),
        EntityKind::Function {
            is_async: true,
            parameter_count: 1,
            return_type: Some("boolean".into()),
        },
        "validateToken()",
        "src/auth/service.ts",
        "typescript",
        DiscoverySource::StaticAnalysis,
    );

    let middleware = ContextEntity::new(
        EntityId::local("src/api/middleware.ts", "authMiddleware"),
        EntityKind::Function {
            is_async: true,
            parameter_count: 2,
            return_type: None,
        },
        "authMiddleware()",
        "src/api/middleware.ts",
        "typescript",
        DiscoverySource::StaticAnalysis,
    );

    let payment = ContextEntity::new(
        EntityId::local("src/payments/checkout.ts", "processPayment"),
        EntityKind::Function {
            is_async: true,
            parameter_count: 1,
            return_type: Some("PaymentResult".into()),
        },
        "processPayment()",
        "src/payments/checkout.ts",
        "typescript",
        DiscoverySource::StaticAnalysis,
    );

    let user_profile = ContextEntity::new(
        EntityId::local("src/users/profile.ts", "getUserProfile"),
        EntityKind::Function {
            is_async: true,
            parameter_count: 1,
            return_type: Some("UserProfile".into()),
        },
        "getUserProfile()",
        "src/users/profile.ts",
        "typescript",
        DiscoverySource::StaticAnalysis,
    );

    let admin_report = ContextEntity::new(
        EntityId::local("src/admin/reports.ts", "generateReport"),
        EntityKind::Function {
            is_async: false,
            parameter_count: 0,
            return_type: Some("Report".into()),
        },
        "generateReport()",
        "src/admin/reports.ts",
        "typescript",
        DiscoverySource::StaticAnalysis,
    );

    let checkout_route = ContextEntity::new(
        EntityId::local("src/routes/checkout.ts", "POST /api/checkout"),
        EntityKind::ApiEndpoint {
            method: "POST".into(),
            route: "/api/checkout".into(),
            handler: "processPayment".into(),
        },
        "POST /api/checkout",
        "src/routes/checkout.ts",
        "typescript",
        DiscoverySource::StaticAnalysis,
    );

    let jira_ticket = ContextEntity::new(
        EntityId::local("jira", "JIRA-AUTH-42"),
        EntityKind::Requirement {
            ticket_id: Some("AUTH-42".into()),
            acceptance_criteria: vec!["OAuth2 migration for validateToken".into()],
        },
        "JIRA-AUTH-42: OAuth2 Migration",
        "jira",
        "jira",
        DiscoverySource::TicketSystem,
    );

    graph.add_entity(auth_svc).unwrap();
    graph.add_entity(middleware).unwrap();
    graph.add_entity(payment).unwrap();
    graph.add_entity(user_profile).unwrap();
    graph.add_entity(admin_report).unwrap();
    graph.add_entity(checkout_route).unwrap();
    graph.add_entity(jira_ticket).unwrap();

    // Edges: middleware imports validateToken
    graph
        .add_relationship(
            &EntityId::local("src/api/middleware.ts", "authMiddleware"),
            &EntityId::local("src/auth/service.ts", "validateToken"),
            ContextEdge::new(
                RelationType::Imports,
                DiscoverySource::StaticAnalysis,
                0.95,
                "imports validateToken directly",
            ),
        )
        .unwrap();

    // payment calls middleware (protected route)
    graph
        .add_relationship(
            &EntityId::local("src/payments/checkout.ts", "processPayment"),
            &EntityId::local("src/api/middleware.ts", "authMiddleware"),
            ContextEdge::new(
                RelationType::Calls,
                DiscoverySource::StaticAnalysis,
                0.80,
                "route uses authMiddleware",
            ),
        )
        .unwrap();

    // userProfile calls middleware
    graph
        .add_relationship(
            &EntityId::local("src/users/profile.ts", "getUserProfile"),
            &EntityId::local("src/api/middleware.ts", "authMiddleware"),
            ContextEdge::new(
                RelationType::Calls,
                DiscoverySource::StaticAnalysis,
                0.85,
                "route uses authMiddleware",
            ),
        )
        .unwrap();

    // checkout route implements processPayment
    graph
        .add_relationship(
            &EntityId::local("src/routes/checkout.ts", "POST /api/checkout"),
            &EntityId::local("src/payments/checkout.ts", "processPayment"),
            ContextEdge::new(
                RelationType::Implements,
                DiscoverySource::StaticAnalysis,
                0.92,
                "POST /api/checkout handler calls processPayment",
            ),
        )
        .unwrap();

    // jira ticket requires validateToken
    graph
        .add_relationship(
            &EntityId::local("jira", "JIRA-AUTH-42"),
            &EntityId::local("src/auth/service.ts", "validateToken"),
            ContextEdge::new(
                RelationType::RequiredBy,
                DiscoverySource::TicketSystem,
                0.70,
                "AUTH-42 requires OAuth2 migration of validateToken",
            ),
        )
        .unwrap();
}

// ─── Handlers ──────────────────────────────────────────────────────

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "contextqa",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

async fn graph_stats(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let graph = state.graph.lock().unwrap();
    let stats = graph.stats();
    Json(serde_json::json!(stats))
}

async fn graph_json(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let graph = state.graph.lock().unwrap();
    match graph.to_json() {
        Ok(json_str) => {
            let value: serde_json::Value = serde_json::from_str(&json_str).unwrap_or_default();
            Ok(Json(value))
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Serializable entity for the API response.
#[derive(Debug, Serialize)]
struct ApiEntity {
    id: String,
    name: String,
    file_path: String,
    kind: String,
}

/// Serializable edge for the API response.
#[derive(Debug, Serialize)]
struct ApiEdge {
    from: String,
    to: String,
    relation: String,
    confidence: f64,
}

async fn graph_entities(State(state): State<Arc<AppState>>) -> Json<Vec<ApiEntity>> {
    let graph = state.graph.lock().unwrap();
    let entities: Vec<ApiEntity> = graph
        .all_entities()
        .into_iter()
        .map(|e| ApiEntity {
            id: e.id.as_str().to_string(),
            name: e.name.clone(),
            file_path: e.file_path.clone(),
            kind: format!("{:?}", e.kind).split('{').next().unwrap_or("Unknown").trim().to_string(),
        })
        .collect();
    Json(entities)
}

async fn graph_edges(State(state): State<Arc<AppState>>) -> Json<Vec<ApiEdge>> {
    let graph = state.graph.lock().unwrap();
    // Use to_json() to get edge snapshot data, then map
    match graph.to_json() {
        Ok(json_str) => {
            let snapshot: serde_json::Value =
                serde_json::from_str(&json_str).unwrap_or_default();
            let edges: Vec<ApiEdge> = snapshot["edges"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .map(|e| ApiEdge {
                    from: e["from"]["raw"].as_str().unwrap_or("").to_string(),
                    to: e["to"]["raw"].as_str().unwrap_or("").to_string(),
                    relation: e["edge"]["relation_type"].as_str().unwrap_or("").to_string(),
                    confidence: e["edge"]["confidence"].as_f64().unwrap_or(0.0),
                })
                .collect();
            Json(edges)
        }
        Err(_) => Json(vec![]),
    }
}

// ─── Ingestion ─────────────────────────────────────────────────────

#[derive(Deserialize)]
struct IngestCodeRequest {
    file_path: String,
    source: String,
    language: String,
}

async fn ingest_code(
    State(state): State<Arc<AppState>>,
    Json(req): Json<IngestCodeRequest>,
) -> Json<serde_json::Value> {
    let events = code_parser::parse_source_code(&req.file_path, &req.source, &req.language);
    let event_count = events.len();

    {
        let mut store = state.event_store.lock().unwrap();
        let mut graph = state.graph.lock().unwrap();
        for event in &events {
            GraphProjection::apply_event(&mut graph, event);
        }
        store.append_batch(events);
    }

    Json(serde_json::json!({
        "status": "ingested",
        "events_created": event_count
    }))
}

#[derive(Deserialize)]
struct IngestDiffRequest {
    file_path: String,
    before: String,
    after: String,
}

async fn ingest_diff(
    State(state): State<Arc<AppState>>,
    Json(req): Json<IngestDiffRequest>,
) -> Json<serde_json::Value> {
    let events = diff_parser::parse_diff(&req.file_path, &req.before, &req.after);
    let event_count = events.len();

    {
        let mut store = state.event_store.lock().unwrap();
        let mut graph = state.graph.lock().unwrap();
        for event in &events {
            GraphProjection::apply_event(&mut graph, event);
        }
        store.append_batch(events);
    }

    Json(serde_json::json!({
        "status": "diff_processed",
        "events_created": event_count
    }))
}

async fn ingest_ticket(
    State(state): State<Arc<AppState>>,
    Json(tickets): Json<Vec<jira_adapter::JiraTicket>>,
) -> Json<serde_json::Value> {
    let mut total_events = 0;

    {
        let mut store = state.event_store.lock().unwrap();
        let mut graph = state.graph.lock().unwrap();
        for ticket in &tickets {
            let events = jira_adapter::ingest_ticket(ticket);
            total_events += events.len();
            for event in &events {
                GraphProjection::apply_event(&mut graph, event);
            }
            store.append_batch(events);
        }
    }

    Json(serde_json::json!({
        "status": "tickets_ingested",
        "events_created": total_events
    }))
}

// ─── Analysis ──────────────────────────────────────────────────────

#[derive(Deserialize)]
struct ImpactRequest {
    changed_entities: Vec<ChangedEntity>,
    min_confidence: Option<f64>,
    max_depth: Option<usize>,
}

#[derive(Deserialize)]
struct ChangedEntity {
    file_path: String,
    symbol: String,
}

async fn impact_analysis(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ImpactRequest>,
) -> Json<serde_json::Value> {
    let graph = state.graph.lock().unwrap();

    let changed: Vec<EntityId> = req
        .changed_entities
        .iter()
        .map(|c| EntityId::local(&c.file_path, &c.symbol))
        .collect();

    let min_conf = req.min_confidence.unwrap_or(0.1);
    let max_depth = req.max_depth.unwrap_or(10);

    let start = std::time::Instant::now();
    let report = analyze_impact(&graph, &changed, min_conf, max_depth);
    let duration = start.elapsed();

    // Record decision trace
    {
        let trace = trace_impact_analysis(
            Uuid::now_v7(),
            graph.stats().entity_count,
            &changed,
            report.direct_impacts.len(),
            report.indirect_impacts.len(),
            report.not_impacted.len(),
            duration.as_millis() as u64,
        );
        let mut trace_store = state.trace_store.lock().unwrap();
        trace_store.record(trace);
    }

    Json(serde_json::to_value(&report).unwrap_or_default())
}

async fn test_intent(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ImpactRequest>,
) -> Json<serde_json::Value> {
    let graph = state.graph.lock().unwrap();

    let changed: Vec<EntityId> = req
        .changed_entities
        .iter()
        .map(|c| EntityId::local(&c.file_path, &c.symbol))
        .collect();

    let min_conf = req.min_confidence.unwrap_or(0.1);
    let max_depth = req.max_depth.unwrap_or(10);

    let report = analyze_impact(&graph, &changed, min_conf, max_depth);
    let intent = generate_test_intent(&report);

    Json(serde_json::to_value(&intent).unwrap_or_default())
}
