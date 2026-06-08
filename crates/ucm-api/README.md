# ucm-api

[![Crates.io](https://img.shields.io/crates/v/ucm-api.svg)](https://crates.io/crates/ucm-api)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](../../LICENSE)

> HTTP API server for the **Unified Context Management (UCM)** engine — built on Axum.

`ucm-api` is the network surface of UCM. It wires the whole workspace together
([`ucm-graph-core`](../ucm-core) + [`ucm-events`](../ucm-events) +
[`ucm-ingest`](../ucm-ingest) + [`ucm-reason`](../ucm-reason)) behind a small,
API-first REST interface, and ships with a seeded demo graph (auth → middleware →
payment) so you can hit `/impact` the moment the server is up.

## Endpoints

| Method | Path | Purpose |
| --- | --- | --- |
| `GET` | `/health` | Liveness check |
| `GET` | `/graph/stats` | Entity / edge counts + average confidence |
| `GET` | `/graph` | Full graph as JSON |
| `GET` | `/graph/entities` | List entities |
| `GET` | `/graph/edges` | List edges with confidence + relation type |
| `POST` | `/ingest/code` | Ingest source code |
| `POST` | `/ingest/diff` | Ingest a unified diff |
| `POST` | `/ingest/ticket` | Ingest a Jira/Linear ticket |
| `POST` | `/impact` | Impact analysis for a changed entity |
| `POST` | `/intent` | Test intent for a change |
| `POST` | `/linear/connect` | Connect a Linear workspace |
| `GET` | `/linear/status` | Linear connection status |
| `POST` | `/ingest/linear` | Pull Linear issues as requirement nodes |

## Run it

```bash
# From the workspace root
cargo run -p ucm-api

# Then, in another terminal (server defaults to port 3001):
curl http://localhost:3001/health
curl http://localhost:3001/graph/stats
curl -X POST http://localhost:3001/impact \
  -H 'content-type: application/json' \
  -d '{"changed_entities":[{"file_path":"src/auth.ts","symbol":"validateToken"}]}'
```

The server reads the `PORT` environment variable (defaults to `3001`),
making it drop-in deployable to Railway, Fly.io, or any container host. CORS is
enabled so the Vercel dashboard can call it directly.

## Architecture notes

- **Axum + Tokio** async server; shared state is `Arc<Mutex<…>>` around the graph,
  event store, and trace store.
- **API-first** — the same JSON contracts power the CLI, the React dashboard, and
  Postman collections.
- **Event-sourced** — every ingestion appends to the log, so the graph is always
  reconstructable.

## License

MIT — see [LICENSE](../../LICENSE).
