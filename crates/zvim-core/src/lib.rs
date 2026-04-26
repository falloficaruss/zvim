//! Shared domain types and primitives for ZVIM.

pub mod config;

pub use config::{
    ConfigLayer, ConfigLoadError, ConfigScope, EditorSettings, EditorSettingsPatch, GitSettings,
    GitSettingsPatch, KeymapSettings, KeymapSettingsPatch, LanguageSettings, LanguageSettingsPatch,
    LspSettings, LspSettingsPatch, PanelDockPosition, ResolvedSettings, Settings, SettingsStore,
    ThemeSettings, ThemeSettingsPatch, UiSettings, UiSettingsPatch,
};

/// Product-wide latency goals to keep performance visible in the architecture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LatencyBudget {
    pub startup_ms: u32,
    pub input_to_paint_ms: u32,
    pub command_ms: u32,
}

impl Default for LatencyBudget {
    fn default() -> Self {
        Self {
            startup_ms: 120,
            input_to_paint_ms: 8,
            command_ms: 4,
        }
    }
}
