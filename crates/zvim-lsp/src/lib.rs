//! Boundary for language-server management, diagnostics, and code actions.

/// Tracks the future direction of LSP integration without coupling it to UI code.
#[derive(Debug, Default)]
pub struct LspRuntime;

impl LspRuntime {
    pub fn is_ready(&self) -> bool {
        true
    }
}
