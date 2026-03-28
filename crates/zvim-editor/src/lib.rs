//! Editing engine for buffers, selections, commands, and viewport behavior.

use zvim_core::LatencyBudget;

/// Minimal editor engine placeholder.
#[derive(Debug, Default)]
pub struct EditorEngine {
    pub budget: LatencyBudget,
}

impl EditorEngine {
    pub fn new() -> Self {
        Self::default()
    }
}

