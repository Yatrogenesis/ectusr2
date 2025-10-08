use anyhow::Result;
use async_trait::async_trait;

pub struct OrchestratorContext {
    pub namespace: Option<String>,
    pub name: Option<String>, // deployment/job name for k8s or similar
    pub model: Option<String>,
}

#[async_trait]
pub trait Orchestrator: Send + Sync {
    async fn status(&self, ctx: &OrchestratorContext) -> Result<String>;
    async fn scale(&self, ctx: &OrchestratorContext, replicas: u32) -> Result<String>;
    async fn ensure_pool(&self, ctx: &OrchestratorContext, spec: &str) -> Result<String>;
}

pub struct LocalOrchestrator;

#[async_trait]
impl Orchestrator for LocalOrchestrator {
    async fn status(&self, _ctx: &OrchestratorContext) -> Result<String> {
        Ok("local: ok".into())
    }
    async fn scale(&self, _ctx: &OrchestratorContext, replicas: u32) -> Result<String> {
        Ok(format!("scaled to {} (local stub)", replicas))
    }
    async fn ensure_pool(&self, _ctx: &OrchestratorContext, _spec: &str) -> Result<String> {
        Ok("pool ensured (local stub)".into())
    }
}

#[cfg(feature = "kubernetes")]
mod kubernetes;

pub fn new_backend(name: &str) -> Box<dyn Orchestrator> {
    match name {
        #[cfg(feature = "kubernetes")]
        s if s.eq_ignore_ascii_case("kubernetes") || s.eq_ignore_ascii_case("k8s") => {
            Box::new(kubernetes::KubeOrchestrator {})
        }
        _ => Box::new(LocalOrchestrator),
    }
}
