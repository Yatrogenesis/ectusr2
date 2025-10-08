use serde_json::{json, Value};
use crate::{api::client::ApiClient, config::Config};
use tracing::{info, warn};

pub fn list() -> Vec<Value> {
    vec![
        json!({"name":"generate_code","description":"Generate production-ready code","inputSchema":{"type":"object"}}),
        json!({"name":"run_qa","description":"Run QA on code","inputSchema":{"type":"object"}}),
        json!({"name":"refactor_code","description":"Apply safe refactorings","inputSchema":{"type":"object"}}),
        json!({"name":"orchestrator_scale","description":"Scale worker pool","inputSchema":{"type":"object"}}),
        json!({"name":"orchestrator_status","description":"Cluster status","inputSchema":{"type":"object"}}),
        json!({"name":"pool_ensure","description":"Ensure model pool","inputSchema":{"type":"object"}}),
        json!({"name":"cost_estimate","description":"Estimate cost","inputSchema":{"type":"object"}}),
        json!({"name":"budget_config","description":"Configure budget policy (read-only in this build; use CLI/env)","inputSchema":{"type":"object"}}),
        json!({"name":"budget_status","description":"Budget status","inputSchema":{"type":"object"}}),
    ]
}

pub async fn call(client: &ApiClient, cfg: &Config, name: &str, args: Value) -> anyhow::Result<String> {
    match name {
        "generate_code" => generate_code(client, args).await,
        "run_qa" => run_qa(client, args).await,
        "refactor_code" => refactor_code(client, args).await,
        "orchestrator_scale" => orchestrator_scale(cfg, args).await,
        "orchestrator_status" => orchestrator_status(cfg, args).await,
        "pool_ensure" => pool_ensure(cfg, args).await,
        "cost_estimate" => cost_estimate(args).await,
        "budget_config" => budget_config(cfg, args).await,
        "budget_status" => budget_status(cfg).await,
        _ => anyhow::bail!("unknown tool: {name}"),
    }
}

async fn generate_code(client: &ApiClient, args: Value) -> anyhow::Result<String> {
    let v = client.post_json("/api/v1/generate", &args).await?;
    Ok(v.to_string())
}

async fn run_qa(client: &ApiClient, args: Value) -> anyhow::Result<String> {
    let v = client.post_json("/api/v1/qa", &args).await?;
    Ok(v.to_string())
}

async fn refactor_code(client: &ApiClient, args: Value) -> anyhow::Result<String> {
    let v = client.post_json("/api/v1/refactor", &args).await?;
    Ok(v.to_string())
}

async fn orchestrator_scale(cfg: &Config, args: Value) -> anyhow::Result<String> {
    use crate::budget::{Resources, estimate_cost, enforce_budget, BudgetPolicy, PolicyKind};
    let backend = args.get("backend").and_then(|v| v.as_str()).unwrap_or(&cfg.orchestrator_backend);
    let replicas = args.get("replicas").and_then(|v| v.as_u64()).unwrap_or(1) as u32;
    let resources: Resources = serde_json::from_value(args.get("resources").cloned().unwrap_or(json!({"cpu":"1","memory":"1Gi"})))?;
    let duration_hours = args.get("duration_hours").and_then(|v| v.as_f64()).unwrap_or(24.0) as f32;
    let budget_enforce = args.get("budget_enforce").and_then(|v| v.as_bool()).unwrap_or(true);
    let override_ok = args.get("override").and_then(|v| v.as_bool()).unwrap_or(false);

    let est = estimate_cost(backend, replicas, &resources, duration_hours);
    info!(backend, replicas, breakdown=%est.breakdown, monthly=%est.monthly_projected_usd, "cost estimated");

    if budget_enforce {
        let policy = BudgetPolicy { monthly_usd_limit: cfg.budget_limit, policy: cfg.budget_policy.as_deref().map(|p| if p.eq_ignore_ascii_case("hard") { PolicyKind::Hard } else { PolicyKind::Soft }) };
        if let Err(msg) = enforce_budget(&policy, est.monthly_projected_usd, override_ok) {
            warn!(%msg, "scale blocked by budget policy");
            return Ok(json!({"ok": false, "reason": msg, "estimate": {"monthly": est.monthly_projected_usd}}).to_string());
        }
    }

    let ctx = crate::orchestrator::OrchestratorContext {
        namespace: args.get("namespace").and_then(|v| v.as_str().map(|s| s.to_string())),
        name: args.get("name").and_then(|v| v.as_str().map(|s| s.to_string())),
        model: args.get("model").and_then(|v| v.as_str().map(|s| s.to_string())),
    };
    let orch = crate::orchestrator::new_backend(backend);
    let res = orch.scale(&ctx, replicas).await?;
    Ok(json!({"ok": true, "action": "scale", "backend": backend, "replicas": replicas, "result": res, "estimate": {"monthly": est.monthly_projected_usd}}).to_string())
}

async fn orchestrator_status(cfg: &Config, args: Value) -> anyhow::Result<String> {
    let backend = args.get("backend").and_then(|v| v.as_str()).unwrap_or(&cfg.orchestrator_backend);
    let ctx = crate::orchestrator::OrchestratorContext {
        namespace: args.get("namespace").and_then(|v| v.as_str().map(|s| s.to_string())),
        name: args.get("name").and_then(|v| v.as_str().map(|s| s.to_string())),
        model: args.get("model").and_then(|v| v.as_str().map(|s| s.to_string())),
    };
    let orch = crate::orchestrator::new_backend(backend);
    let status = orch.status(&ctx).await?;
    Ok(json!({"backend": backend, "status": status}).to_string())
}

async fn pool_ensure(cfg: &Config, args: Value) -> anyhow::Result<String> {
    let backend = args.get("backend").and_then(|v| v.as_str()).unwrap_or(&cfg.orchestrator_backend);
    let spec = args.get("spec").cloned().unwrap_or(json!({})).to_string();
    let ctx = crate::orchestrator::OrchestratorContext {
        namespace: args.get("namespace").and_then(|v| v.as_str().map(|s| s.to_string())),
        name: args.get("name").and_then(|v| v.as_str().map(|s| s.to_string())),
        model: args.get("model").and_then(|v| v.as_str().map(|s| s.to_string())),
    };
    let orch = crate::orchestrator::new_backend(backend);
    let res = orch.ensure_pool(&ctx, &spec).await?;
    Ok(json!({"backend": backend, "result": res}).to_string())
}

async fn cost_estimate(args: Value) -> anyhow::Result<String> {
    use crate::budget::{Resources, estimate_cost};
    let backend = args.get("backend").and_then(|v| v.as_str()).unwrap_or("kubernetes");
    let replicas = args.get("replicas").and_then(|v| v.as_u64()).unwrap_or(1) as u32;
    let resources: Resources = serde_json::from_value(args.get("resources").cloned().unwrap_or(json!({"cpu":"1","memory":"1Gi"})))?;
    let duration_hours = args.get("duration_hours").and_then(|v| v.as_f64()).unwrap_or(24.0) as f32;
    let est = estimate_cost(backend, replicas, &resources, duration_hours);
    Ok(json!({"backend": backend, "replicas": replicas, "hourly_total_usd": est.hourly_total_usd, "monthly_projected_usd": est.monthly_projected_usd, "breakdown": est.breakdown}).to_string())
}

async fn budget_config(cfg: &Config, _args: Value) -> anyhow::Result<String> {
    Ok(json!({"policy": cfg.budget_policy, "monthly_usd_limit": cfg.budget_limit}).to_string())
}

async fn budget_status(cfg: &Config) -> anyhow::Result<String> {
    Ok(json!({"month_to_date_usd": 0.0, "projected_eom_usd": cfg.budget_limit.unwrap_or(0.0), "headroom_usd": cfg.budget_limit.unwrap_or(0.0)}).to_string())
}
