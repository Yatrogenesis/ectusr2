# ectusr2

Ectus-R MCP server in Rust. Implements MCP over stdio with tools for code generation, QA and refactoring; includes multi-orchestration (Kubernetes adapter) and budget monitoring, optional Prometheus metrics, and a Helm chart.

## Build & Run

- Release build: `cargo build --release`
- Run (stdio MCP): `target/release/ectusr2`
- Env:
  - `ECTUS_R_API_URL` (default `http://localhost:8000`)
  - `ECTUS_R_API_KEY` (optional)
  - `ORCH_BACKEND` (`kubernetes`|`local`|...)
  - `BUDGET_MONTHLY_USD_LIMIT`, `BUDGET_POLICY` (`hard`|`soft`)
  - `RUST_LOG` (e.g., `info,ectusr2=debug`)

## Metrics (optional)

- Build with feature: `cargo build --release --features metrics`
- Set `METRICS_ADDR`, e.g.: `0.0.0.0:9900`
- `GET /metrics` (Prometheus text format)

## Kubernetes Orchestration (optional)

- Build with feature: `cargo build --release --features kubernetes`
- Tools (via MCP):
  - `orchestrator_scale` `{ backend, namespace, name, replicas, resources, budget_enforce }`
  - `orchestrator_status` `{ backend, namespace, name }`
  - `pool_ensure` `{ backend, namespace, name, spec }` (server-side apply Deployment)

## Docker

- Build image: `docker build -t ghcr.io/Yatrogenesis/ectusr2:latest .`
- Run: `docker run --rm -e METRICS_ADDR=0.0.0.0:9900 -p 9900:9900 ghcr.io/Yatrogenesis/ectusr2:latest`

## Helm

- `kubectl create ns aion`
- `helm upgrade --install ectusr2 ./deploy/helm/ectusr2 -n aion`
- values.yaml highlights:
  - `image.repository`: `ghcr.io/Yatrogenesis/ectusr2`
  - `metrics.enabled/port`
  - `orchestrator.backend/namespace/workersName`
  - `orchestrator.hpa.*` (optional)

## MCP JSON Examples

- tools/list request:
```
{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}
```
- tools/call generate_code:
```
{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"generate_code","arguments":{"requirements":"Create an HTTP server with /health","language":"rust","framework":"axum","quality_level":0.95,"include_tests":true,"include_docs":true}}}
```
- tools/call orchestrator_scale (k8s):
```
{"jsonrpc":"2.0","id":10,"method":"tools/call","params":{"name":"orchestrator_scale","arguments":{"backend":"kubernetes","namespace":"aion","name":"ectusr2-workers","replicas":5,"resources":{"cpu":"1","memory":"1Gi"},"budget_enforce":true}}}
```

## CI

- GitHub Actions: build, test, clippy, fmt; audit/deny/sbom jobs.

## License

MIT
