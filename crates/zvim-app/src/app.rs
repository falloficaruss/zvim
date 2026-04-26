use std::env;
use std::path::{Path, PathBuf};

use zvim_core::{ConfigLoadError, ConfigScope, ResolvedSettings, SettingsStore};
use zvim_editor::EditorEngine;
use zvim_lsp::LspRuntime;

#[derive(Debug)]
pub struct ZvimApp {
    session: AppSession,
}

impl ZvimApp {
    pub fn bootstrap() -> Result<Self, BootstrapError> {
        let workspace_root = env::current_dir().map_err(|source| ConfigLoadError::Read {
            path: ".".to_string(),
            source,
        })?;
        let config_paths = discover_config_paths(&workspace_root);

        let mut store = SettingsStore::new();
        let mut loaded_layers = Vec::new();

        for (scope, path) in [
            (ConfigScope::User, config_paths.user),
            (ConfigScope::Workspace, config_paths.workspace),
            (ConfigScope::Project, config_paths.project),
        ] {
            if store.load_optional_toml_layer_from_path(scope, &path)? {
                loaded_layers.push((scope, path));
            }
        }

        let settings = store.resolve();
        let session = AppSession {
            workspace: WorkspaceState::new(workspace_root, loaded_layers, settings),
            editor: EditorEngine::new(),
            lsp: LspRuntime::default(),
        };

        Ok(Self { session })
    }

    pub fn session(&self) -> &AppSession {
        &self.session
    }

    pub fn render_boot_report(&self) -> String {
        let workspace = &self.session.workspace;
        let loaded_layers = if workspace.loaded_layers.is_empty() {
            "none".to_string()
        } else {
            workspace
                .loaded_layers
                .iter()
                .map(|(scope, path)| format!("{scope:?}:{}", path.display()))
                .collect::<Vec<_>>()
                .join(", ")
        };

        [
            "ZVIM bootstrap".to_string(),
            format!("workspace_root = {}", workspace.root.display()),
            format!("loaded_layers = {loaded_layers}"),
            format!("editor.tab_size = {}", workspace.settings.editor.tab_size),
            format!("editor.soft_wrap = {}", workspace.settings.editor.soft_wrap),
            format!(
                "ui.sidebar_visible = {}",
                workspace.settings.ui.sidebar_visible
            ),
            format!("ui.show_tabs = {}", workspace.settings.ui.show_tabs),
            format!(
                "ui.theme.font_family = {}",
                workspace.settings.ui.theme.font_family
            ),
            format!(
                "keymap.modal_editing = {}",
                workspace.settings.keymap.modal_editing
            ),
            format!("git.inline_blame = {}", workspace.settings.git.inline_blame),
            format!(
                "lsp.format_on_save = {}",
                workspace.settings.lsp.format_on_save
            ),
            format!(
                "editor.command_budget_ms = {}",
                self.session.editor.budget.command_ms
            ),
            format!("lsp.runtime_state = {}", self.session.lsp.state()),
            format!("lsp.runtime_ready = {}", self.session.lsp.is_ready()),
        ]
        .join("\n")
    }
}

#[derive(Debug)]
pub struct AppSession {
    pub workspace: WorkspaceState,
    pub editor: EditorEngine,
    pub lsp: LspRuntime,
}

#[derive(Debug)]
pub struct WorkspaceState {
    pub root: PathBuf,
    pub loaded_layers: Vec<(ConfigScope, PathBuf)>,
    pub settings: ResolvedSettings,
}

impl WorkspaceState {
    pub fn new(
        root: PathBuf,
        loaded_layers: Vec<(ConfigScope, PathBuf)>,
        settings: ResolvedSettings,
    ) -> Self {
        Self {
            root,
            loaded_layers,
            settings,
        }
    }
}

#[derive(Debug)]
pub struct ConfigPaths {
    pub user: PathBuf,
    pub workspace: PathBuf,
    pub project: PathBuf,
}

pub fn discover_config_paths(workspace_root: &Path) -> ConfigPaths {
    let user = env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .unwrap_or_else(|| workspace_root.join(".zvim-user"))
        .join("zvim")
        .join("settings.toml");

    let workspace = workspace_root.join(".zvim").join("settings.toml");
    let project = workspace_root.join("zvim.toml");

    ConfigPaths {
        user,
        workspace,
        project,
    }
}

#[derive(Debug)]
pub enum BootstrapError {
    Config(ConfigLoadError),
    UiLaunch(String),
}

impl From<ConfigLoadError> for BootstrapError {
    fn from(value: ConfigLoadError) -> Self {
        Self::Config(value)
    }
}

impl std::fmt::Display for BootstrapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Config(error) => write!(f, "{error}"),
            Self::UiLaunch(message) => write!(f, "failed to launch UI: {message}"),
        }
    }
}

impl std::error::Error for BootstrapError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Config(error) => Some(error),
            Self::UiLaunch(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discover_config_paths_prefers_workspace_conventions() {
        let root = Path::new("/tmp/example");
        let paths = discover_config_paths(root);

        assert_eq!(
            paths.workspace,
            PathBuf::from("/tmp/example/.zvim/settings.toml")
        );
        assert_eq!(paths.project, PathBuf::from("/tmp/example/zvim.toml"));
    }

    #[test]
    fn boot_report_mentions_loaded_layers() {
        let settings = SettingsStore::new().resolve();
        let session = AppSession {
            workspace: WorkspaceState::new(
                PathBuf::from("/tmp/example"),
                vec![(
                    ConfigScope::Workspace,
                    PathBuf::from("/tmp/example/.zvim/settings.toml"),
                )],
                settings,
            ),
            editor: EditorEngine::new(),
            lsp: LspRuntime::default(),
        };
        let app = ZvimApp { session };

        let report = app.render_boot_report();

        assert!(report.contains("loaded_layers = Workspace:/tmp/example/.zvim/settings.toml"));
        assert!(report.contains("lsp.runtime_state = not_started"));
        assert!(report.contains("lsp.runtime_ready = false"));
    }
}
