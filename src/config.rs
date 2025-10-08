use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub api_url: String,
    pub api_key: Option<String>,
    pub orchestrator_backend: String,
    pub budget_limit: Option<f32>,
    pub budget_policy: Option<String>,
}

impl Config {
    pub fn from_cli(cli: impl Into<CliShim>) -> anyhow::Result<Self> {
        let c = cli.into();
        // Merge env with CLI (CLI wins if provided)
        let api_url = if !c.api_url.is_empty() { c.api_url } else { env::var("ECTUS_R_API_URL").unwrap_or_else(|_| "http://localhost:8000".into()) };
        let api_key = c.api_key.or_else(|| env::var("ECTUS_R_API_KEY").ok());
        let orchestrator_backend = if !c.orchestrator_backend.is_empty() { c.orchestrator_backend } else { env::var("ORCH_BACKEND").unwrap_or_else(|_| "kubernetes".into()) };
        let budget_limit = c.budget_limit.or_else(|| env::var("BUDGET_MONTHLY_USD_LIMIT").ok().and_then(|s| s.parse::<f32>().ok()));
        let budget_policy = c.budget_policy.or_else(|| env::var("BUDGET_POLICY").ok());

        Ok(Self { api_url, api_key, orchestrator_backend, budget_limit, budget_policy })
    }
}

// Shim to avoid clap in this module
#[derive(Debug, Clone)]
pub struct CliShim {
    pub api_url: String,
    pub api_key: Option<String>,
    pub orchestrator_backend: String,
    pub budget_limit: Option<f32>,
    pub budget_policy: Option<String>,
}

impl From<crate::Cli> for CliShim {
    fn from(c: crate::Cli) -> Self {
        Self {
            api_url: c.api_url,
            api_key: c.api_key,
            orchestrator_backend: c.orchestrator_backend,
            budget_limit: c.budget_limit,
            budget_policy: c.budget_policy,
        }
    }
}
