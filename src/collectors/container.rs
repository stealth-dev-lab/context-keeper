//! Container collector - detects running Docker/Podman containers

use super::traits::Collector;
use crate::config::Config;
use crate::context::{ContainerInfo, Context};

/// Container collector
#[derive(Debug, Default)]
pub struct ContainerCollector;

impl Collector for ContainerCollector {
    fn name(&self) -> &'static str {
        "container"
    }

    fn is_enabled(&self, _config: &Config) -> bool {
        // Always enabled - containers are a core feature
        true
    }

    fn collect(&self, config: &Config, ctx: &mut Context) {
        ctx.containers = collect_containers(config);
    }
}

/// Collect running containers
fn collect_containers(config: &Config) -> Vec<ContainerInfo> {
    let mut containers = Vec::new();

    let runtime = config
        .containers
        .as_ref()
        .and_then(|c| c.runtime.as_deref())
        .unwrap_or("podman");

    if let Ok(output) = std::process::Command::new(runtime)
        .args(["ps", "--format", "{{.Names}}\\t{{.Status}}"])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let parts: Vec<&str> = line.split('\t').collect();
                if parts.len() >= 2 {
                    containers.push(ContainerInfo {
                        name: parts[0].to_string(),
                        status: parts[1].to_string(),
                        runtime: runtime.to_string(),
                    });
                }
            }
        }
    }

    containers
}
