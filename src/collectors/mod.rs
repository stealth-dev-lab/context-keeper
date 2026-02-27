//! Context collectors module
//!
//! This module provides a trait-based collector system for gathering
//! development context from various sources.

mod traits;
mod build;
mod container;
mod git;
mod history;
mod adb;
mod workstate;

pub use traits::Collector;
pub use build::BuildCollector;
pub use container::ContainerCollector;
pub use git::GitCollector;
pub use history::HistoryCollector;
pub use adb::AdbCollector;
pub use workstate::{
    WorkStateCollector,
    save_work_state_to_file,
    collect_working_files,
};

use crate::config::Config;
use crate::context::Context;

/// Get all available collectors
pub fn default_collectors() -> Vec<Box<dyn Collector>> {
    vec![
        Box::new(BuildCollector),
        Box::new(ContainerCollector),
        Box::new(GitCollector),
        Box::new(HistoryCollector),
        Box::new(AdbCollector),
        Box::new(WorkStateCollector),
    ]
}

/// Collect all context data using the collector registry
pub fn collect_context(config: &Config) -> Context {
    collect_context_with(config, &default_collectors())
}

/// Collect context data using a custom set of collectors
pub fn collect_context_with(config: &Config, collectors: &[Box<dyn Collector>]) -> Context {
    let mut ctx = Context::default();

    // Set project info from config
    if let Some(project) = &config.project {
        ctx.project_name = project.name.clone().unwrap_or_default();
        ctx.project_type = project.project_type.clone().unwrap_or_default();
    }

    // Set hints from config
    if let Some(hints) = &config.hints {
        ctx.hints = hints.default.clone().unwrap_or_default();
    }

    // Run enabled collectors
    for collector in collectors {
        if collector.is_enabled(config) {
            collector.collect(config, &mut ctx);
        }
    }

    ctx
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_collectors() {
        let collectors = default_collectors();
        assert_eq!(collectors.len(), 6);

        let names: Vec<&str> = collectors.iter().map(|c| c.name()).collect();
        assert!(names.contains(&"build"));
        assert!(names.contains(&"container"));
        assert!(names.contains(&"git"));
        assert!(names.contains(&"history"));
        assert!(names.contains(&"adb"));
        assert!(names.contains(&"workstate"));
    }

    #[test]
    fn test_collect_context_empty_config() {
        let config = Config::default();
        let ctx = collect_context(&config);

        // Should not panic with empty config
        assert!(ctx.project_name.is_empty());
    }
}
