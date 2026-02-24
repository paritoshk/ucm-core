//! UCM API server — Axum REST endpoints for the unified context graph.
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
//! - POST /linear/connect     — connect Linear workspace via API key
//! - GET  /linear/status      — check Linear connection status
//! - POST /ingest/linear      — fetch and ingest issues from Linear

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

use ucm_core::edge::{RelationType, UcmEdge};
use ucm_core::entity::*;
use ucm_core::graph::UcmGraph;
use ucm_events::projection::GraphProjection;
use ucm_events::store::EventStore;
use ucm_ingest::{code_parser, diff_parser, jira_adapter, linear_adapter};
use ucm_observe::trace::{trace_impact_analysis, TraceStore};
use ucm_reason::ambiguity::enrich_with_ambiguities;
use ucm_reason::impact::analyze_impact;
use ucm_reason::intent::generate_test_intent;

/// Shared application state.
struct AppState {
    graph: Mutex<UcmGraph>,
    event_store: Mutex<EventStore>,
    trace_store: Mutex<TraceStore>,
    linear_api_key: Mutex<Option<String>>,
    linear_workspace: Mutex<Option<String>>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // Seed the demo graph on startup
    let mut graph = UcmGraph::new();
    seed_demo_graph(&mut graph);
    tracing::info!("Seeded demo graph: {:?}", graph.stats());

    let state = Arc::new(AppState {
        graph: Mutex::new(graph),
        event_store: Mutex::new(EventStore::new()),
        trace_store: Mutex::new(TraceStore::new()),
        linear_api_key: Mutex::new(None),
        linear_workspace: Mutex::new(None),
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
        .route("/linear/connect", post(connect_linear))
        .route("/linear/status", get(linear_status))
        .route("/ingest/linear", post(ingest_linear))
        .layer(CorsLayer::permissive())
        .with_state(state);

    // PORT env var for Railway compatibility (defaults to 3001)
    let port = env::var("PORT").unwrap_or_else(|_| "3001".to_string());
    let bind_addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&bind_addr).await.unwrap();
    tracing::info!("UCM API server listening on http://{bind_addr}");
    axum::serve(listener, app).await.unwrap();
}

// ─── Demo Graph Seeder ─────────────────────────────────────────────

/// Seeds the graph with the auth→middleware→payment demo scenario.
/// Includes 7 entities and 5 edges matching the dashboard's demo data.
fn seed_demo_graph(graph: &mut UcmGraph) {
    // Entities
    let auth_svc = UcmEntity::new(
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

    let middleware = UcmEntity::new(
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

    let payment = UcmEntity::new(
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

    let user_profile = UcmEntity::new(
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

    let admin_report = UcmEntity::new(
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

    let checkout_route = UcmEntity::new(
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

    let jira_ticket = UcmEntity::new(
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
            UcmEdge::new(
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
            UcmEdge::new(
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
            UcmEdge::new(
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
            UcmEdge::new(
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
            UcmEdge::new(
                RelationType::RequiredBy,
                DiscoverySource::TicketSystem,
                0.70,
                "AUTH-42 requires OAuth2 migration of validateToken",
            ),
        )
        .unwrap();

    // Low-confidence edge from API traffic — triggers ambiguity detection.
    // API logs suggest generateReport may call getUserProfile, but only seen
    // in 8 out of 100 sampled requests (possibly a logging artifact).
    graph
        .add_relationship(
            &EntityId::local("src/admin/reports.ts", "generateReport"),
            &EntityId::local("src/users/profile.ts", "getUserProfile"),
            UcmEdge::new(
                RelationType::Calls,
                DiscoverySource::ApiTraffic,
                0.45,
                "API traffic logs show occasional calls (8/100 samples) — possible logging artifact",
            ),
        )
        .unwrap();

    // Conflicting requirement: Jira says OAuth2 only, but API traffic still
    // shows JWT bearer tokens in production — requirement drift.
    graph
        .add_relationship(
            &EntityId::local("jira", "JIRA-AUTH-42"),
            &EntityId::local("src/api/middleware.ts", "authMiddleware"),
            UcmEdge::new(
                RelationType::Implements,
                DiscoverySource::TicketSystem,
                0.50,
                "JIRA-AUTH-42 says migrate to OAuth2, but middleware still uses JWT bearer tokens",
            ),
        )
        .unwrap();
}

// ─── Handlers ──────────────────────────────────────────────────────

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "ucm",
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
            kind: format!("{:?}", e.kind)
                .split('{')
                .next()
                .unwrap_or("Unknown")
                .trim()
                .to_string(),
        })
        .collect();
    Json(entities)
}

async fn graph_edges(State(state): State<Arc<AppState>>) -> Json<Vec<ApiEdge>> {
    let graph = state.graph.lock().unwrap();
    // Use to_json() to get edge snapshot data, then map
    match graph.to_json() {
        Ok(json_str) => {
            let snapshot: serde_json::Value = serde_json::from_str(&json_str).unwrap_or_default();
            let edges: Vec<ApiEdge> = snapshot["edges"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .map(|e| ApiEdge {
                    from: e["from"]["raw"].as_str().unwrap_or("").to_string(),
                    to: e["to"]["raw"].as_str().unwrap_or("").to_string(),
                    relation: e["edge"]["relation_type"]
                        .as_str()
                        .unwrap_or("")
                        .to_string(),
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
    use ucm_core::event::EventPayload;
    let events = code_parser::parse_source_code(&req.file_path, &req.source, &req.language);

    let entities_discovered = events
        .iter()
        .filter(|e| matches!(&e.payload, EventPayload::EntityDiscovered { .. }))
        .count();
    let relationships_detected = events
        .iter()
        .filter(|e| matches!(&e.payload, EventPayload::DependencyLinked { .. }))
        .count();

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
        "entities_discovered": entities_discovered,
        "relationships_detected": relationships_detected,
        // Note: relationships_detected counts edges emitted by the parser.
        // Edges pointing to entities not yet in the graph are held pending
        // until those entities are ingested (ingest all files for full graph).
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
    let mut report = analyze_impact(&graph, &changed, min_conf, max_depth);
    enrich_with_ambiguities(&mut report, &graph, 0.60);
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

    let mut report = analyze_impact(&graph, &changed, min_conf, max_depth);
    enrich_with_ambiguities(&mut report, &graph, 0.60);
    let intent = generate_test_intent(&report);

    Json(serde_json::to_value(&intent).unwrap_or_default())
}

// ─── Linear Integration ─────────────────────────────────────────

#[derive(Deserialize)]
struct ConnectLinearRequest {
    api_key: String,
}

async fn connect_linear(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ConnectLinearRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Validate the key by querying Linear's API
    let client = reqwest::Client::new();
    let res = client
        .post("https://api.linear.app/graphql")
        .header("Authorization", &req.api_key)
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "query": "{ viewer { id name } organization { name } }"
        }))
        .send()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    if !res.status().is_success() {
        return Ok(Json(serde_json::json!({
            "connected": false,
            "error": "Invalid API key"
        })));
    }

    let body: serde_json::Value = res.json().await.map_err(|_| StatusCode::BAD_GATEWAY)?;
    let workspace = body["data"]["organization"]["name"]
        .as_str()
        .unwrap_or("Unknown")
        .to_string();

    *state.linear_api_key.lock().unwrap() = Some(req.api_key);
    *state.linear_workspace.lock().unwrap() = Some(workspace.clone());

    tracing::info!("Linear connected: workspace={workspace}");

    Ok(Json(serde_json::json!({
        "connected": true,
        "workspace": workspace
    })))
}

async fn linear_status(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let has_key = state.linear_api_key.lock().unwrap().is_some();
    let workspace = state.linear_workspace.lock().unwrap().clone();

    Json(serde_json::json!({
        "connected": has_key,
        "workspace": workspace
    }))
}

async fn ingest_linear(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let api_key = state.linear_api_key.lock().unwrap().clone();
    let api_key = match api_key {
        Some(k) => k,
        None => {
            return Ok(Json(serde_json::json!({
                "error": "Linear not connected. Call /linear/connect first."
            })));
        }
    };

    // Fetch issues from Linear GraphQL API
    let client = reqwest::Client::new();
    let res = client
        .post("https://api.linear.app/graphql")
        .header("Authorization", &api_key)
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "query": r#"{
                issues(first: 50, orderBy: updatedAt) {
                    nodes {
                        identifier
                        title
                        description
                        priority
                        state { name }
                        labels { nodes { name } }
                        assignee { name }
                        url
                    }
                }
            }"#
        }))
        .send()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    let body: serde_json::Value = res.json().await.map_err(|_| StatusCode::BAD_GATEWAY)?;

    // Map GraphQL response to LinearIssue structs
    let nodes = body["data"]["issues"]["nodes"]
        .as_array()
        .cloned()
        .unwrap_or_default();

    let issues: Vec<linear_adapter::LinearIssue> = nodes
        .iter()
        .map(|node| linear_adapter::LinearIssue {
            identifier: node["identifier"].as_str().unwrap_or("").to_string(),
            title: node["title"].as_str().unwrap_or("").to_string(),
            description: node["description"].as_str().unwrap_or("").to_string(),
            state: node["state"]["name"].as_str().unwrap_or("").to_string(),
            priority: format!("{}", node["priority"].as_f64().unwrap_or(0.0) as u8),
            labels: node["labels"]["nodes"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|l| l["name"].as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default(),
            assignee: node["assignee"]["name"].as_str().map(|s| s.to_string()),
            url: node["url"].as_str().map(|s| s.to_string()),
        })
        .collect();

    let issues_count = issues.len();
    let mut total_events = 0;

    {
        let mut store = state.event_store.lock().unwrap();
        let mut graph = state.graph.lock().unwrap();
        for issue in &issues {
            let events = linear_adapter::ingest_linear_issue(issue);
            total_events += events.len();
            for event in &events {
                GraphProjection::apply_event(&mut graph, event);
            }
            store.append_batch(events);
        }
    }

    tracing::info!("Ingested {issues_count} Linear issues ({total_events} events)");

    Ok(Json(serde_json::json!({
        "status": "ingested",
        "issues_count": issues_count,
        "events_created": total_events
    })))
}
