#[cfg(feature = "kubernetes")]
pub struct KubeOrchestrator;

#[cfg(feature = "kubernetes")]
#[async_trait::async_trait]
impl crate::orchestrator::Orchestrator for KubeOrchestrator {
    async fn status(&self, ctx: &crate::orchestrator::OrchestratorContext) -> anyhow::Result<String> {
        use kube::{Client, Api, api::ListParams};
        use k8s_openapi::api::apps::v1::Deployment;
        let ns = ctx.namespace.as_deref().unwrap_or("default");
        let client = Client::try_default().await?;
        let api: Api<Deployment> = Api::namespaced(client, ns);
        let list = api.list(&ListParams::default()).await?;
        let mut out = Vec::new();
        for d in list { let name=d.metadata.name.unwrap_or_default(); out.push(name); }
        Ok(format!("deployments in {}: {}", ns, out.join(", ")))
    }
    async fn scale(&self, ctx: &crate::orchestrator::OrchestratorContext, replicas: u32) -> anyhow::Result<String> {
        use kube::{Client, Api, api::{Patch, PatchParams}};
        use k8s_openapi::api::apps::v1::Deployment;
        let ns = ctx.namespace.as_deref().unwrap_or("default");
        let name = ctx.name.as_deref().unwrap_or("ectusr2-workers");
        let client = Client::try_default().await?;
        let api: Api<Deployment> = Api::namespaced(client, ns);
        let patch = serde_json::json!({"spec": {"replicas": replicas as i32}});
        let pp = PatchParams::apply("ectusr2").force();
        let _ = api.patch(name, &pp, &Patch::Apply(&patch)).await?;
        Ok(format!("scaled {} to {} in {}", name, replicas, ns))
    }
    async fn ensure_pool(&self, _ctx: &crate::orchestrator::OrchestratorContext, _spec: &str) -> anyhow::Result<String> {
        Ok("ensure_pool not implemented for k8s yet".into())
    }
}
#[cfg(feature = "kubernetes")]
use serde_json::Value as Json;

#[cfg(feature = "kubernetes")]
impl KubeOrchestrator {
    async fn ensure_deployment(ns: &str, name: &str, spec: &str) -> anyhow::Result<String> {
        use kube::{Client, Api, api::{Patch, PatchParams}};
        use k8s_openapi::api::apps::v1::Deployment;
        let client = Client::try_default().await?;
        let api: Api<Deployment> = Api::namespaced(client, ns);
        // Accept prebuilt Deployment JSON or a minimal spec { image, replicas, labels, resources }
        let parsed: Json = serde_json::from_str(spec).unwrap_or(Json::Object(Default::default()));
        let replicas = parsed.get("replicas").and_then(|v| v.as_u64()).unwrap_or(1) as i32;
        let image = parsed.get("image").and_then(|v| v.as_str()).unwrap_or("busybox:stable");
        let labels = parsed.get("labels").cloned().unwrap_or(serde_json::json!({"app": name}));
        // Very small patch to set replicas and a single container with image
        let patch = serde_json::json!({
           "apiVersion":"apps/v1",
           "kind":"Deployment",
           "metadata": {"name": name, "labels": labels},
           "spec": {
             "replicas": replicas,
             "selector": {"matchLabels": {"app": name}},
             "template": {
               "metadata": {"labels": {"app": name}},
               "spec": {"containers": [{"name": name, "image": image}]}
             }
           }
        });
        let pp = PatchParams::apply("ectusr2").force();
        let _ = api.patch(name, &pp, &Patch::Apply(&patch)).await?;
        Ok(format!("ensured deployment {} in {} (replicas {})", name, ns, replicas))
    }
}

