//! Configuration primitives for layered ZVIM settings.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

/// Ordered scope precedence for settings resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ConfigScope {
    Default,
    User,
    Workspace,
    Project,
}

impl ConfigScope {
    pub const ALL: [Self; 4] = [Self::Default, Self::User, Self::Workspace, Self::Project];
}

/// A partial settings payload attached to a specific scope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigLayer {
    pub scope: ConfigScope,
    pub settings: Settings,
}

impl ConfigLayer {
    pub fn new(scope: ConfigScope, settings: Settings) -> Self {
        Self { scope, settings }
    }
}

/// A store that resolves settings by merging layers in precedence order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsStore {
    layers: BTreeMap<ConfigScope, Settings>,
}

#[derive(Debug)]
pub enum ConfigLoadError {
    Read {
        path: String,
        source: std::io::Error,
    },
    ParseToml {
        path: Option<String>,
        source: toml::de::Error,
    },
    Invalid {
        path: Option<String>,
        message: String,
    },
}

impl ConfigLoadError {
    pub fn path(&self) -> Option<&str> {
        match self {
            Self::Read { path, .. } => Some(path.as_str()),
            Self::ParseToml { path, .. } | Self::Invalid { path, .. } => path.as_deref(),
        }
    }

    pub fn is_not_found(&self) -> bool {
        matches!(
            self,
            Self::Read {
                source,
                ..
            } if source.kind() == std::io::ErrorKind::NotFound
        )
    }

    fn with_path(self, path: impl Into<String>) -> Self {
        let path = path.into();
        match self {
            Self::Read { path, source } => Self::Read { path, source },
            Self::ParseToml {
                path: existing,
                source,
            } => Self::ParseToml {
                path: existing.or(Some(path)),
                source,
            },
            Self::Invalid {
                path: existing,
                message,
            } => Self::Invalid {
                path: existing.or(Some(path)),
                message,
            },
        }
    }
}

impl std::fmt::Display for ConfigLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Read { path, source } => write!(f, "failed to read config file {path}: {source}"),
            Self::ParseToml {
                path: Some(path),
                source,
            } => write!(f, "failed to parse TOML config {path}: {source}"),
            Self::ParseToml { path: None, source } => {
                write!(f, "failed to parse TOML config: {source}")
            }
            Self::Invalid {
                path: Some(path),
                message,
            } => write!(f, "invalid config in {path}: {message}"),
            Self::Invalid {
                path: None,
                message,
            } => write!(f, "invalid config: {message}"),
        }
    }
}

impl std::error::Error for ConfigLoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Read { source, .. } => Some(source),
            Self::ParseToml { source, .. } => Some(source),
            Self::Invalid { .. } => None,
        }
    }
}

impl Default for SettingsStore {
    fn default() -> Self {
        let mut layers = BTreeMap::new();
        layers.insert(ConfigScope::Default, Settings::defaults_layer());
        Self { layers }
    }
}

impl SettingsStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_layer(&mut self, layer: ConfigLayer) {
        self.layers.insert(layer.scope, layer.settings);
    }

    pub fn clear_layer(&mut self, scope: ConfigScope) {
        if scope != ConfigScope::Default {
            self.layers.remove(&scope);
        }
    }

    pub fn layer(&self, scope: ConfigScope) -> Option<&Settings> {
        self.layers.get(&scope)
    }

    pub fn resolve(&self) -> ResolvedSettings {
        let mut resolved = ResolvedSettings::default();
        for scope in ConfigScope::ALL {
            if let Some(layer) = self.layers.get(&scope) {
                resolved.apply(layer);
            }
        }
        resolved
    }

    pub fn load_toml_layer_from_path<P: AsRef<Path>>(
        &mut self,
        scope: ConfigScope,
        path: P,
    ) -> Result<(), ConfigLoadError> {
        let settings = Settings::load_toml_from_path(path)?;
        self.set_layer(ConfigLayer::new(scope, settings));
        Ok(())
    }

    pub fn load_optional_toml_layer_from_path<P: AsRef<Path>>(
        &mut self,
        scope: ConfigScope,
        path: P,
    ) -> Result<bool, ConfigLoadError> {
        match Settings::load_toml_from_path(path) {
            Ok(settings) => {
                self.set_layer(ConfigLayer::new(scope, settings));
                Ok(true)
            }
            Err(error) if error.is_not_found() => Ok(false),
            Err(error) => Err(error),
        }
    }
}

/// Partial settings payload. `None` means "inherit from a lower-precedence scope".
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "snake_case")]
pub struct Settings {
    pub editor: Option<EditorSettingsPatch>,
    pub ui: Option<UiSettingsPatch>,
    pub keymap: Option<KeymapSettingsPatch>,
    pub git: Option<GitSettingsPatch>,
    pub lsp: Option<LspSettingsPatch>,
    pub languages: BTreeMap<String, LanguageSettingsPatch>,
}

impl Settings {
    pub fn defaults_layer() -> Self {
        Self {
            editor: Some(EditorSettings::default().into_patch()),
            ui: Some(UiSettings::default().into_patch()),
            keymap: Some(KeymapSettings::default().into_patch()),
            git: Some(GitSettings::default().into_patch()),
            lsp: Some(LspSettings::default().into_patch()),
            languages: BTreeMap::new(),
        }
    }

    pub fn from_toml_str(input: &str) -> Result<Self, ConfigLoadError> {
        let settings: Self = toml::from_str(input)
            .map_err(|source| ConfigLoadError::ParseToml { path: None, source })?;
        settings.validate()?;
        Ok(settings)
    }

    pub fn load_toml_from_path<P: AsRef<Path>>(path: P) -> Result<Self, ConfigLoadError> {
        let path = path.as_ref();
        let contents = fs::read_to_string(path).map_err(|source| ConfigLoadError::Read {
            path: path.display().to_string(),
            source,
        })?;
        Self::from_toml_str(&contents).map_err(|error| error.with_path(path.display().to_string()))
    }

    pub fn validate(&self) -> Result<(), ConfigLoadError> {
        if let Some(editor) = &self.editor {
            validate_optional_tab_size(editor.tab_size, "editor.tab_size")?;
        }

        if let Some(ui) = &self.ui {
            if let Some(theme) = &ui.theme {
                validate_optional_non_empty_string(
                    theme.font_family.as_deref(),
                    "ui.theme.font_family",
                )?;
                validate_optional_positive_u16(theme.font_size_px, "ui.theme.font_size_px")?;
                validate_optional_positive_u16(
                    theme.line_height_percent,
                    "ui.theme.line_height_percent",
                )?;
            }
        }

        if let Some(keymap) = &self.keymap {
            validate_optional_non_empty_string(keymap.leader_key.as_deref(), "keymap.leader_key")?;
            validate_optional_positive_u16(keymap.chord_timeout_ms, "keymap.chord_timeout_ms")?;
        }

        if let Some(git) = &self.git {
            validate_optional_positive_u16(git.auto_refresh_ms, "git.auto_refresh_ms")?;
        }

        for (language, patch) in &self.languages {
            validate_optional_non_empty_string(
                patch.formatter.as_deref(),
                &format!("languages.{language}.formatter"),
            )?;
            validate_optional_tab_size(patch.tab_size, &format!("languages.{language}.tab_size"))?;
        }

        Ok(())
    }
}

fn invalid_config(message: impl Into<String>) -> ConfigLoadError {
    ConfigLoadError::Invalid {
        path: None,
        message: message.into(),
    }
}

fn validate_optional_tab_size(value: Option<u8>, field: &str) -> Result<(), ConfigLoadError> {
    match value {
        Some(0) => Err(invalid_config(format!("{field} must be greater than 0"))),
        _ => Ok(()),
    }
}

fn validate_optional_positive_u16(value: Option<u16>, field: &str) -> Result<(), ConfigLoadError> {
    match value {
        Some(0) => Err(invalid_config(format!("{field} must be greater than 0"))),
        _ => Ok(()),
    }
}

fn validate_optional_non_empty_string(
    value: Option<&str>,
    field: &str,
) -> Result<(), ConfigLoadError> {
    match value {
        Some(value) if value.trim().is_empty() => {
            Err(invalid_config(format!("{field} must not be empty")))
        }
        _ => Ok(()),
    }
}

/// Fully resolved settings produced after layering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedSettings {
    pub editor: EditorSettings,
    pub ui: UiSettings,
    pub keymap: KeymapSettings,
    pub git: GitSettings,
    pub lsp: LspSettings,
    pub languages: BTreeMap<String, LanguageSettings>,
}

impl Default for ResolvedSettings {
    fn default() -> Self {
        Self {
            editor: EditorSettings::default(),
            ui: UiSettings::default(),
            keymap: KeymapSettings::default(),
            git: GitSettings::default(),
            lsp: LspSettings::default(),
            languages: BTreeMap::new(),
        }
    }
}

impl ResolvedSettings {
    fn apply(&mut self, layer: &Settings) {
        if let Some(editor) = &layer.editor {
            self.editor.apply(editor);
        }
        if let Some(ui) = &layer.ui {
            self.ui.apply(ui);
        }
        if let Some(keymap) = &layer.keymap {
            self.keymap.apply(keymap);
        }
        if let Some(git) = &layer.git {
            self.git.apply(git);
        }
        if let Some(lsp) = &layer.lsp {
            self.lsp.apply(lsp);
        }
        for (language, patch) in &layer.languages {
            self.languages
                .entry(language.clone())
                .or_default()
                .apply(patch);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct EditorSettings {
    pub tab_size: u8,
    pub use_spaces: bool,
    pub soft_wrap: bool,
    pub show_line_numbers: bool,
    pub relative_line_numbers: bool,
    pub cursor_blink: bool,
    pub restore_last_cursor_position: bool,
}

impl Default for EditorSettings {
    fn default() -> Self {
        Self {
            tab_size: 4,
            use_spaces: true,
            soft_wrap: false,
            show_line_numbers: true,
            relative_line_numbers: true,
            cursor_blink: true,
            restore_last_cursor_position: true,
        }
    }
}

impl EditorSettings {
    pub fn apply(&mut self, patch: &EditorSettingsPatch) {
        if let Some(tab_size) = patch.tab_size {
            self.tab_size = tab_size;
        }
        if let Some(use_spaces) = patch.use_spaces {
            self.use_spaces = use_spaces;
        }
        if let Some(soft_wrap) = patch.soft_wrap {
            self.soft_wrap = soft_wrap;
        }
        if let Some(show_line_numbers) = patch.show_line_numbers {
            self.show_line_numbers = show_line_numbers;
        }
        if let Some(relative_line_numbers) = patch.relative_line_numbers {
            self.relative_line_numbers = relative_line_numbers;
        }
        if let Some(cursor_blink) = patch.cursor_blink {
            self.cursor_blink = cursor_blink;
        }
        if let Some(restore_last_cursor_position) = patch.restore_last_cursor_position {
            self.restore_last_cursor_position = restore_last_cursor_position;
        }
    }

    pub fn into_patch(self) -> EditorSettingsPatch {
        EditorSettingsPatch {
            tab_size: Some(self.tab_size),
            use_spaces: Some(self.use_spaces),
            soft_wrap: Some(self.soft_wrap),
            show_line_numbers: Some(self.show_line_numbers),
            relative_line_numbers: Some(self.relative_line_numbers),
            cursor_blink: Some(self.cursor_blink),
            restore_last_cursor_position: Some(self.restore_last_cursor_position),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "snake_case")]
pub struct EditorSettingsPatch {
    pub tab_size: Option<u8>,
    pub use_spaces: Option<bool>,
    pub soft_wrap: Option<bool>,
    pub show_line_numbers: Option<bool>,
    pub relative_line_numbers: Option<bool>,
    pub cursor_blink: Option<bool>,
    pub restore_last_cursor_position: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct UiSettings {
    pub sidebar_visible: bool,
    pub panel_dock_position: PanelDockPosition,
    pub compact_mode: bool,
    pub show_tabs: bool,
    pub theme: ThemeSettings,
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            sidebar_visible: true,
            panel_dock_position: PanelDockPosition::Left,
            compact_mode: false,
            show_tabs: true,
            theme: ThemeSettings::default(),
        }
    }
}

impl UiSettings {
    pub fn apply(&mut self, patch: &UiSettingsPatch) {
        if let Some(sidebar_visible) = patch.sidebar_visible {
            self.sidebar_visible = sidebar_visible;
        }
        if let Some(panel_dock_position) = patch.panel_dock_position {
            self.panel_dock_position = panel_dock_position;
        }
        if let Some(compact_mode) = patch.compact_mode {
            self.compact_mode = compact_mode;
        }
        if let Some(show_tabs) = patch.show_tabs {
            self.show_tabs = show_tabs;
        }
        if let Some(theme) = &patch.theme {
            self.theme.apply(theme);
        }
    }

    pub fn into_patch(self) -> UiSettingsPatch {
        UiSettingsPatch {
            sidebar_visible: Some(self.sidebar_visible),
            panel_dock_position: Some(self.panel_dock_position),
            compact_mode: Some(self.compact_mode),
            show_tabs: Some(self.show_tabs),
            theme: Some(self.theme.into_patch()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "snake_case")]
pub struct UiSettingsPatch {
    pub sidebar_visible: Option<bool>,
    pub panel_dock_position: Option<PanelDockPosition>,
    pub compact_mode: Option<bool>,
    pub show_tabs: Option<bool>,
    pub theme: Option<ThemeSettingsPatch>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ThemeSettings {
    pub font_family: String,
    pub font_size_px: u16,
    pub line_height_percent: u16,
}

impl Default for ThemeSettings {
    fn default() -> Self {
        Self {
            font_family: "Iosevka".to_string(),
            font_size_px: 14,
            line_height_percent: 140,
        }
    }
}

impl ThemeSettings {
    pub fn apply(&mut self, patch: &ThemeSettingsPatch) {
        if let Some(font_family) = &patch.font_family {
            self.font_family = font_family.clone();
        }
        if let Some(font_size_px) = patch.font_size_px {
            self.font_size_px = font_size_px;
        }
        if let Some(line_height_percent) = patch.line_height_percent {
            self.line_height_percent = line_height_percent;
        }
    }

    pub fn into_patch(self) -> ThemeSettingsPatch {
        ThemeSettingsPatch {
            font_family: Some(self.font_family),
            font_size_px: Some(self.font_size_px),
            line_height_percent: Some(self.line_height_percent),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "snake_case")]
pub struct ThemeSettingsPatch {
    pub font_family: Option<String>,
    pub font_size_px: Option<u16>,
    pub line_height_percent: Option<u16>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PanelDockPosition {
    Left,
    Right,
    Bottom,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct KeymapSettings {
    pub modal_editing: bool,
    pub leader_key: String,
    pub allow_chord_timeout: bool,
    pub chord_timeout_ms: u16,
}

impl Default for KeymapSettings {
    fn default() -> Self {
        Self {
            modal_editing: true,
            leader_key: "space".to_string(),
            allow_chord_timeout: true,
            chord_timeout_ms: 800,
        }
    }
}

impl KeymapSettings {
    pub fn apply(&mut self, patch: &KeymapSettingsPatch) {
        if let Some(modal_editing) = patch.modal_editing {
            self.modal_editing = modal_editing;
        }
        if let Some(leader_key) = &patch.leader_key {
            self.leader_key = leader_key.clone();
        }
        if let Some(allow_chord_timeout) = patch.allow_chord_timeout {
            self.allow_chord_timeout = allow_chord_timeout;
        }
        if let Some(chord_timeout_ms) = patch.chord_timeout_ms {
            self.chord_timeout_ms = chord_timeout_ms;
        }
    }

    pub fn into_patch(self) -> KeymapSettingsPatch {
        KeymapSettingsPatch {
            modal_editing: Some(self.modal_editing),
            leader_key: Some(self.leader_key),
            allow_chord_timeout: Some(self.allow_chord_timeout),
            chord_timeout_ms: Some(self.chord_timeout_ms),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "snake_case")]
pub struct KeymapSettingsPatch {
    pub modal_editing: Option<bool>,
    pub leader_key: Option<String>,
    pub allow_chord_timeout: Option<bool>,
    pub chord_timeout_ms: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct GitSettings {
    pub gutter_diff: bool,
    pub inline_diff: bool,
    pub inline_blame: bool,
    pub auto_refresh_ms: u16,
}

impl Default for GitSettings {
    fn default() -> Self {
        Self {
            gutter_diff: true,
            inline_diff: true,
            inline_blame: false,
            auto_refresh_ms: 400,
        }
    }
}

impl GitSettings {
    pub fn apply(&mut self, patch: &GitSettingsPatch) {
        if let Some(gutter_diff) = patch.gutter_diff {
            self.gutter_diff = gutter_diff;
        }
        if let Some(inline_diff) = patch.inline_diff {
            self.inline_diff = inline_diff;
        }
        if let Some(inline_blame) = patch.inline_blame {
            self.inline_blame = inline_blame;
        }
        if let Some(auto_refresh_ms) = patch.auto_refresh_ms {
            self.auto_refresh_ms = auto_refresh_ms;
        }
    }

    pub fn into_patch(self) -> GitSettingsPatch {
        GitSettingsPatch {
            gutter_diff: Some(self.gutter_diff),
            inline_diff: Some(self.inline_diff),
            inline_blame: Some(self.inline_blame),
            auto_refresh_ms: Some(self.auto_refresh_ms),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "snake_case")]
pub struct GitSettingsPatch {
    pub gutter_diff: Option<bool>,
    pub inline_diff: Option<bool>,
    pub inline_blame: Option<bool>,
    pub auto_refresh_ms: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct LspSettings {
    pub diagnostics: bool,
    pub hover: bool,
    pub completion: bool,
    pub format_on_save: bool,
}

impl Default for LspSettings {
    fn default() -> Self {
        Self {
            diagnostics: true,
            hover: true,
            completion: true,
            format_on_save: false,
        }
    }
}

impl LspSettings {
    pub fn apply(&mut self, patch: &LspSettingsPatch) {
        if let Some(diagnostics) = patch.diagnostics {
            self.diagnostics = diagnostics;
        }
        if let Some(hover) = patch.hover {
            self.hover = hover;
        }
        if let Some(completion) = patch.completion {
            self.completion = completion;
        }
        if let Some(format_on_save) = patch.format_on_save {
            self.format_on_save = format_on_save;
        }
    }

    pub fn into_patch(self) -> LspSettingsPatch {
        LspSettingsPatch {
            diagnostics: Some(self.diagnostics),
            hover: Some(self.hover),
            completion: Some(self.completion),
            format_on_save: Some(self.format_on_save),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "snake_case")]
pub struct LspSettingsPatch {
    pub diagnostics: Option<bool>,
    pub hover: Option<bool>,
    pub completion: Option<bool>,
    pub format_on_save: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct LanguageSettings {
    pub formatter: Option<String>,
    pub format_on_save: bool,
    pub tab_size: Option<u8>,
    pub use_spaces: Option<bool>,
}

impl Default for LanguageSettings {
    fn default() -> Self {
        Self {
            formatter: None,
            format_on_save: false,
            tab_size: None,
            use_spaces: None,
        }
    }
}

impl LanguageSettings {
    pub fn apply(&mut self, patch: &LanguageSettingsPatch) {
        if let Some(formatter) = &patch.formatter {
            self.formatter = Some(formatter.clone());
        }
        if let Some(format_on_save) = patch.format_on_save {
            self.format_on_save = format_on_save;
        }
        if let Some(tab_size) = patch.tab_size {
            self.tab_size = Some(tab_size);
        }
        if let Some(use_spaces) = patch.use_spaces {
            self.use_spaces = Some(use_spaces);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "snake_case")]
pub struct LanguageSettingsPatch {
    pub formatter: Option<String>,
    pub format_on_save: Option<bool>,
    pub tab_size: Option<u8>,
    pub use_spaces: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_layer_is_present() {
        let store = SettingsStore::new();
        let resolved = store.resolve();

        assert!(resolved.keymap.modal_editing);
        assert!(resolved.ui.sidebar_visible);
        assert_eq!(resolved.editor.tab_size, 4);
    }

    #[test]
    fn higher_precedence_layers_override_only_requested_editor_fields() {
        let mut store = SettingsStore::new();
        store.set_layer(ConfigLayer::new(
            ConfigScope::User,
            Settings {
                editor: Some(EditorSettingsPatch {
                    tab_size: Some(2),
                    ..EditorSettingsPatch::default()
                }),
                ..Settings::default()
            },
        ));
        store.set_layer(ConfigLayer::new(
            ConfigScope::Workspace,
            Settings {
                editor: Some(EditorSettingsPatch {
                    soft_wrap: Some(true),
                    ..EditorSettingsPatch::default()
                }),
                ..Settings::default()
            },
        ));

        let resolved = store.resolve();
        assert_eq!(resolved.editor.tab_size, 2);
        assert!(resolved.editor.soft_wrap);
        assert!(resolved.editor.use_spaces);
    }

    #[test]
    fn nested_theme_settings_can_be_partially_overridden() {
        let mut store = SettingsStore::new();
        store.set_layer(ConfigLayer::new(
            ConfigScope::User,
            Settings {
                ui: Some(UiSettingsPatch {
                    theme: Some(ThemeSettingsPatch {
                        font_size_px: Some(16),
                        ..ThemeSettingsPatch::default()
                    }),
                    ..UiSettingsPatch::default()
                }),
                ..Settings::default()
            },
        ));

        let resolved = store.resolve();
        assert_eq!(resolved.ui.theme.font_size_px, 16);
        assert_eq!(resolved.ui.theme.font_family, "Iosevka");
    }

    #[test]
    fn language_settings_merge_by_language_key() {
        let mut store = SettingsStore::new();
        let mut user_languages = BTreeMap::new();
        user_languages.insert(
            "rust".to_string(),
            LanguageSettingsPatch {
                formatter: Some("rustfmt".to_string()),
                ..LanguageSettingsPatch::default()
            },
        );
        store.set_layer(ConfigLayer::new(
            ConfigScope::User,
            Settings {
                languages: user_languages,
                ..Settings::default()
            },
        ));

        let mut workspace_languages = BTreeMap::new();
        workspace_languages.insert(
            "rust".to_string(),
            LanguageSettingsPatch {
                format_on_save: Some(true),
                ..LanguageSettingsPatch::default()
            },
        );
        workspace_languages.insert(
            "toml".to_string(),
            LanguageSettingsPatch {
                tab_size: Some(2),
                ..LanguageSettingsPatch::default()
            },
        );
        store.set_layer(ConfigLayer::new(
            ConfigScope::Workspace,
            Settings {
                languages: workspace_languages,
                ..Settings::default()
            },
        ));

        let resolved = store.resolve();
        let rust = resolved.languages.get("rust").expect("rust settings");
        let toml = resolved.languages.get("toml").expect("toml settings");

        assert_eq!(rust.formatter.as_deref(), Some("rustfmt"));
        assert!(rust.format_on_save);
        assert_eq!(toml.tab_size, Some(2));
    }

    #[test]
    fn settings_can_be_parsed_from_toml() {
        let parsed = Settings::from_toml_str(
            r#"
            [editor]
            tab_size = 2
            soft_wrap = true

            [ui]
            sidebar_visible = false

            [ui.theme]
            font_size_px = 16

            [languages.rust]
            formatter = "rustfmt"
            format_on_save = true
            "#,
        )
        .expect("valid TOML settings");

        let editor = parsed.editor.expect("editor patch");
        let ui = parsed.ui.expect("ui patch");
        let rust = parsed.languages.get("rust").expect("rust patch");

        assert_eq!(editor.tab_size, Some(2));
        assert_eq!(editor.soft_wrap, Some(true));
        assert_eq!(ui.sidebar_visible, Some(false));
        assert_eq!(ui.theme.and_then(|theme| theme.font_size_px), Some(16));
        assert_eq!(rust.formatter.as_deref(), Some("rustfmt"));
        assert_eq!(rust.format_on_save, Some(true));
    }

    #[test]
    fn loaded_toml_layer_merges_with_defaults() {
        let mut store = SettingsStore::new();
        store.set_layer(ConfigLayer::new(
            ConfigScope::User,
            Settings::from_toml_str(
                r#"
                [editor]
                tab_size = 2

                [git]
                inline_blame = true
                "#,
            )
            .expect("valid TOML settings"),
        ));

        let resolved = store.resolve();

        assert_eq!(resolved.editor.tab_size, 2);
        assert!(resolved.git.inline_blame);
        assert!(resolved.git.inline_diff);
    }

    #[test]
    fn invalid_editor_tab_size_is_rejected() {
        let error = Settings::from_toml_str(
            r#"
            [editor]
            tab_size = 0
            "#,
        )
        .expect_err("invalid settings should fail");

        assert_eq!(error.path(), None);
        assert_eq!(
            error.to_string(),
            "invalid config: editor.tab_size must be greater than 0"
        );
    }

    #[test]
    fn parse_errors_loaded_from_path_include_path_context() {
        let path =
            std::env::temp_dir().join(format!("zvim-invalid-config-{}.toml", std::process::id()));
        fs::write(&path, "[editor\n tab_size = 4").expect("write invalid settings");

        let error = Settings::load_toml_from_path(&path).expect_err("invalid TOML should fail");
        let path_str = path.to_string_lossy().into_owned();

        assert_eq!(error.path(), Some(path_str.as_str()));
        assert!(error.to_string().contains(path_str.as_str()));

        fs::remove_file(path).expect("cleanup temp config");
    }
}
