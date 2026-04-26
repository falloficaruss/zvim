//! Boundary for language-server management, diagnostics, and code actions.

/// Tracks the future direction of LSP integration without coupling it to UI code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LspRuntimeState {
    #[default]
    NotStarted,
    Starting,
    Ready,
    Failed,
}

impl std::fmt::Display for LspRuntimeState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::NotStarted => "not_started",
            Self::Starting => "starting",
            Self::Ready => "ready",
            Self::Failed => "failed",
        };
        f.write_str(value)
    }
}

#[derive(Debug, Default)]
pub struct LspRuntime {
    state: LspRuntimeState,
}

impl LspRuntime {
    pub fn state(&self) -> LspRuntimeState {
        self.state
    }

    pub fn is_ready(&self) -> bool {
        matches!(self.state, LspRuntimeState::Ready)
    }
}
