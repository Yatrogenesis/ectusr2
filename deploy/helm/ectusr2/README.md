# ectusr2 Helm Chart

Deploys the ectusr2 MCP server with optional Prometheus metrics and RBAC for Kubernetes orchestration.

## Install

- Create namespace (optional):
  - `kubectl create ns aion`
- Install chart:
  - `helm upgrade --install ectusr2 ./deploy/helm/ectusr2 -n aion`

## Values

- `image.repository` (string): container image repo
- `image.tag` (string): tag
- `metrics.enabled` (bool): expose /metrics
- `metrics.port` (int): metrics port (default 9900)
- `orchestrator.backend` (string): `kubernetes` recommended
- `orchestrator.namespace` (string): namespace to operate
- `orchestrator.workersName` (string): name of workers Deployment to scale
- `rbac.create` (bool): create SA/Role/RoleBinding
- `serviceAccount.create` (bool): create SA
- `resources` (map): pod resources for ectusr2
- `orchestrator.hpa.enabled` (bool): manage HPA for workers
- `orchestrator.hpa.minReplicas` (int)
- `orchestrator.hpa.maxReplicas` (int)
- `orchestrator.hpa.targetCPUUtilizationPercentage` (int)

## Notes

- The chart deploys the ectusr2 server and RBAC; worker Deployments are assumed to be created by the server (ensure `workersName` matches).
- Set env `METRICS_ADDR` automatically when `metrics.enabled=true`.
