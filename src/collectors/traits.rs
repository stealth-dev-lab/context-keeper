//! Collector trait definition

use crate::config::Config;
use crate::context::Context;

/// Trait for context collectors
///
/// Each collector is responsible for gathering a specific type of
/// development context (build targets, containers, git repos, etc.)
pub trait Collector: Send + Sync {
    /// Returns the collector's name for logging and debugging
    fn name(&self) -> &'static str;

    /// Check if this collector is enabled based on configuration
    fn is_enabled(&self, config: &Config) -> bool;

    /// Collect context data and update the Context struct
    fn collect(&self, config: &Config, ctx: &mut Context);
}
