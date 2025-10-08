use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Resources {
    pub cpu: String,      // e.g., "500m" or "1"
    pub memory: String,   // e.g., "512Mi" or "2Gi"
    pub gpu: Option<String>,
}

#[derive(Debug, Clone)]
pub struct BudgetPolicy {
    pub monthly_usd_limit: Option<f32>,
    pub policy: Option<PolicyKind>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyKind { Hard, Soft }

pub struct EstimateResult {
    pub hourly_total_usd: f32,
    pub monthly_projected_usd: f32,
    pub breakdown: String,
}

pub fn estimate_cost(backend: &str, replicas: u32, resources: &Resources, hours: f32) -> EstimateResult {
    // Simple static rates as fallback (USD per hour per unit)
    // CPU: per vCPU hour; Memory: per GiB hour; GPU flat add if present
    let (cpu_rate, mem_rate, gpu_rate) = match backend.to_lowercase().as_str() {
        "ecs" => (0.040, 0.005, 1.50),
        "kubernetes" => (0.035, 0.004, 1.40),
        "cloud_run" | "aca" => (0.045, 0.006, 1.80),
        "asg" | "mig" | "vmss" => (0.030, 0.004, 1.30),
        _ => (0.040, 0.005, 1.50),
    };

    let cpu_cores = parse_cpu(&resources.cpu);
    let mem_gib = parse_mem_gib(&resources.memory);
    let gpu_count = resources.gpu.as_deref().and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);

    let hourly_per_replica = cpu_cores * cpu_rate + mem_gib * mem_rate + (gpu_count as f32) * gpu_rate;
    let hourly_total = hourly_per_replica * replicas as f32;
    let monthly_projected = hourly_total * 24.0 * 30.0; // simple projection

    EstimateResult {
        hourly_total_usd: (hourly_total * hours / hours.max(1.0)),
        monthly_projected_usd: monthly_projected,
        breakdown: format!(
            "replicas={} cpu={}cores mem={}Gi gpu={} -> perReplica=${:.4}/h",
            replicas, cpu_cores, mem_gib, gpu_count, hourly_per_replica
        ),
    }
}

pub fn enforce_budget(policy: &BudgetPolicy, projected_monthly: f32, override_ok: bool) -> Result<(), String> {
    if let Some(limit) = policy.monthly_usd_limit {
        if projected_monthly > limit {
            match policy.policy {
                Some(PolicyKind::Hard) => return Err(format!("budget hard-limit exceeded (limit=${:.2}, projected=${:.2})", limit, projected_monthly)),
                Some(PolicyKind::Soft) => {
                    if !override_ok { return Err(format!("budget soft-limit exceeded; override required (limit=${:.2}, projected=${:.2})", limit, projected_monthly)); }
                }
                None => {}
            }
        }
    }
    Ok(())
}

fn parse_cpu(s: &str) -> f32 {
    if let Some(stripped) = s.strip_suffix('m') { // millicores
        return stripped.parse::<f32>().unwrap_or(0.0) / 1000.0;
    }
    s.parse::<f32>().unwrap_or(0.0)
}

fn parse_mem_gib(s: &str) -> f32 {
    let lower = s.trim().to_ascii_lowercase();
    // Tebibytes/Terabytes → GiB
    if let Some(v) = lower.strip_suffix("tib") { return v.trim().parse::<f32>().unwrap_or(0.0) * 1024.0; }
    if let Some(v) = lower.strip_suffix("ti") { return v.trim().parse::<f32>().unwrap_or(0.0) * 1024.0; }
    if let Some(v) = lower.strip_suffix("tb") { return v.trim().parse::<f32>().unwrap_or(0.0) * 1024.0; }
    // Gibibytes/Gigabytes → GiB
    if let Some(v) = lower.strip_suffix("gib") { return v.trim().parse::<f32>().unwrap_or(0.0); }
    if let Some(v) = lower.strip_suffix("gi") { return v.trim().parse::<f32>().unwrap_or(0.0); }
    if let Some(v) = lower.strip_suffix("gb") { return v.trim().parse::<f32>().unwrap_or(0.0); }
    if let Some(v) = lower.strip_suffix("g") { return v.trim().parse::<f32>().unwrap_or(0.0); }
    // Mebibytes/Megabytes → GiB
    if let Some(v) = lower.strip_suffix("mib") { return v.trim().parse::<f32>().unwrap_or(0.0) / 1024.0; }
    if let Some(v) = lower.strip_suffix("mi") { return v.trim().parse::<f32>().unwrap_or(0.0) / 1024.0; }
    if let Some(v) = lower.strip_suffix("mb") { return v.trim().parse::<f32>().unwrap_or(0.0) / 1024.0; }
    if let Some(v) = lower.strip_suffix("m") { return v.trim().parse::<f32>().unwrap_or(0.0) / 1024.0; }
    lower.parse::<f32>().unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cpu() {
        assert!((parse_cpu("1000m") - 1.0).abs() < 1e-6);
        assert!((parse_cpu("500m") - 0.5).abs() < 1e-6);
        assert!((parse_cpu("2") - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_parse_mem() {
        assert!((parse_mem_gib("1024Mi") - 1.0).abs() < 1e-6);
        assert!((parse_mem_gib("1Gi") - 1.0).abs() < 1e-6);
        assert!((parse_mem_gib("2048MB") - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_estimate_and_enforce_soft() {
        let res = Resources { cpu: "1".into(), memory: "1Gi".into(), gpu: None };
        let est = estimate_cost("kubernetes", 5, &res, 24.0);
        assert!(est.monthly_projected_usd > 0.0);
        let pol = BudgetPolicy { monthly_usd_limit: Some(est.monthly_projected_usd - 1.0), policy: Some(PolicyKind::Soft) };
        // soft should error unless override
        assert!(enforce_budget(&pol, est.monthly_projected_usd, false).is_err());
        assert!(enforce_budget(&pol, est.monthly_projected_usd, true).is_ok());
    }

    #[test]
    fn test_enforce_hard() {
        let pol = BudgetPolicy { monthly_usd_limit: Some(10.0), policy: Some(PolicyKind::Hard) };
        assert!(enforce_budget(&pol, 100.0, false).is_err());
        assert!(enforce_budget(&pol, 5.0, false).is_ok());
    }
}


