# ectusr2

Ectus-R MCP server in Rust. Implements MCP over stdio with tools for code generation, QA and refactoring, plus multi-orchestration and budget monitoring stubs.

- Build: `cargo build --release`
- Run: wired via stdio from an MCP client
- Env: `ECTUS_R_API_URL`, `ECTUS_R_API_KEY`, `ORCH_BACKEND`, `BUDGET_MONTHLY_USD_LIMIT`, `BUDGET_POLICY`, `RUST_LOG`
