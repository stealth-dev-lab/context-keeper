//! Context output command (--context)

use crate::collectors::collect_context;
use crate::config::read_config;
use crate::formatters::format_context_markdown;

/// Run the context output command
pub fn run_context_command(level: &str) {
    let config = read_config();
    let context = collect_context(&config);
    println!("{}", format_context_markdown(&context, level));
}
