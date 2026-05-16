use serde::Deserialize;
use std::{
    collections::HashSet,
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateCatalog {
    pub templates: Vec<SessionTemplate>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub layout: TemplateLayout,
    pub panes: Vec<TemplatePane>,
    pub focus_pane: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplatePane {
    pub command: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateLayout {
    Single,
    Columns,
    Rows,
}

#[derive(Debug, Deserialize)]
struct TemplateConfig {
    templates: Option<Vec<RawTemplate>>,
}

#[derive(Debug, Deserialize)]
struct RawTemplate {
    id: Option<String>,
    name: Option<String>,
    description: Option<String>,
    layout: Option<String>,
    focus_pane: Option<usize>,
    panes: Option<Vec<RawPane>>,
}

#[derive(Debug, Deserialize)]
struct RawPane {
    command: Option<String>,
}

impl TemplateCatalog {
    pub fn load() -> Self {
        let mut catalog = Self::builtins();

        if let Some(path) = user_templates_path() {
            catalog.load_user_templates(&path);
        }

        catalog
    }

    fn builtins() -> Self {
        Self {
            templates: builtin_templates(),
            warnings: Vec::new(),
        }
    }

    fn load_user_templates(&mut self, path: &Path) {
        match fs::read_to_string(path) {
            Ok(contents) => self.merge_user_config(&contents, &path.display().to_string()),
            Err(err) if err.kind() == ErrorKind::NotFound => {}
            Err(err) => self.warnings.push(format!(
                "Could not read template config {}: {}",
                path.display(),
                err
            )),
        }
    }

    fn merge_user_config(&mut self, contents: &str, source: &str) {
        let config = match toml::from_str::<TemplateConfig>(contents) {
            Ok(config) => config,
            Err(err) => {
                self.warnings.push(format!(
                    "Could not parse template config {}: {}",
                    source, err
                ));
                return;
            }
        };

        let mut seen: HashSet<String> = self
            .templates
            .iter()
            .map(|template| template.id.clone())
            .collect();

        for raw in config.templates.unwrap_or_default() {
            match validate_user_template(raw, &seen) {
                Ok(template) => {
                    seen.insert(template.id.clone());
                    self.templates.push(template);
                }
                Err(warning) => self.warnings.push(warning),
            }
        }
    }

    #[cfg(test)]
    fn from_config_str(contents: &str) -> Self {
        let mut catalog = Self::builtins();
        catalog.merge_user_config(contents, "test");
        catalog
    }
}

impl SessionTemplate {
    fn new(
        id: &str,
        name: &str,
        description: &str,
        layout: TemplateLayout,
        panes: Vec<TemplatePane>,
        focus_pane: usize,
    ) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            layout,
            panes,
            focus_pane,
        }
    }

    pub fn is_terminal(&self) -> bool {
        self.id == "terminal"
    }

    pub fn split_percent(&self) -> Option<u8> {
        (self.id == "nvim-codex" || self.id == "nvim-gemini").then_some(70)
    }

    pub fn pane_summary(&self) -> String {
        let separator = match self.layout {
            TemplateLayout::Single => "",
            TemplateLayout::Columns => " | ",
            TemplateLayout::Rows => " / ",
        };

        self.panes
            .iter()
            .map(TemplatePane::label)
            .collect::<Vec<_>>()
            .join(separator)
    }
}

impl TemplatePane {
    fn shell() -> Self {
        Self {
            command: String::new(),
        }
    }

    fn command(command: &str) -> Self {
        Self {
            command: command.to_string(),
        }
    }

    pub fn label(&self) -> &str {
        if self.command.trim().is_empty() {
            "shell"
        } else {
            self.command.as_str()
        }
    }
}

impl TemplateLayout {
    fn from_config(value: &str) -> Option<Self> {
        match value.trim() {
            "single" => Some(Self::Single),
            "columns" => Some(Self::Columns),
            "rows" => Some(Self::Rows),
            _ => None,
        }
    }

    pub fn split_flag(self) -> Option<&'static str> {
        match self {
            Self::Single => None,
            Self::Columns => Some("-h"),
            Self::Rows => Some("-v"),
        }
    }

    pub fn tmux_even_layout(self) -> Option<&'static str> {
        match self {
            Self::Single => None,
            Self::Columns => Some("even-horizontal"),
            Self::Rows => Some("even-vertical"),
        }
    }
}

pub fn user_templates_path() -> Option<PathBuf> {
    user_templates_path_from_env(
        std::env::var("XDG_CONFIG_HOME").ok().as_deref(),
        std::env::var("HOME").ok().as_deref(),
    )
}

fn user_templates_path_from_env(
    xdg_config_home: Option<&str>,
    home: Option<&str>,
) -> Option<PathBuf> {
    if let Some(xdg_config_home) = xdg_config_home
        && !xdg_config_home.trim().is_empty()
    {
        return Some(PathBuf::from(xdg_config_home).join("trex/templates.toml"));
    }

    home.filter(|home| !home.trim().is_empty())
        .map(|home| PathBuf::from(home).join(".config/trex/templates.toml"))
}

fn builtin_templates() -> Vec<SessionTemplate> {
    vec![
        SessionTemplate::new(
            "terminal",
            "Terminal",
            "One shell pane in the selected directory",
            TemplateLayout::Single,
            vec![TemplatePane::shell()],
            0,
        ),
        SessionTemplate::new(
            "two-columns",
            "Two Columns",
            "Two side-by-side shell panes",
            TemplateLayout::Columns,
            vec![TemplatePane::shell(), TemplatePane::shell()],
            0,
        ),
        SessionTemplate::new(
            "two-rows",
            "Two Rows",
            "Two stacked shell panes",
            TemplateLayout::Rows,
            vec![TemplatePane::shell(), TemplatePane::shell()],
            0,
        ),
        SessionTemplate::new(
            "nvim-codex",
            "nvim + Codex",
            "Codex on the left, nvim on the right",
            TemplateLayout::Columns,
            vec![
                TemplatePane::command("codex"),
                TemplatePane::command("nvim"),
            ],
            0,
        ),
        SessionTemplate::new(
            "nvim-gemini",
            "nvim + Gemini",
            "Gemini on the left, nvim on the right",
            TemplateLayout::Columns,
            vec![
                TemplatePane::command("gemini"),
                TemplatePane::command("nvim"),
            ],
            0,
        ),
    ]
}

fn validate_user_template(
    raw: RawTemplate,
    seen_ids: &HashSet<String>,
) -> Result<SessionTemplate, String> {
    let id = required_field(raw.id, "id")?;
    if !is_valid_template_id(&id) {
        return Err(format!(
            "Skipped template {id}: id must use letters, digits, '-', or '_'"
        ));
    }

    if seen_ids.contains(&id) {
        return Err(format!("Skipped template {id}: duplicate template id"));
    }

    let name = required_field(raw.name, "name")?;
    let description = raw.description.unwrap_or_default();
    let layout_value = required_field(raw.layout, "layout")?;
    let layout = TemplateLayout::from_config(&layout_value)
        .ok_or_else(|| format!("Skipped template {id}: unsupported layout {layout_value}"))?;

    let panes = raw
        .panes
        .unwrap_or_default()
        .into_iter()
        .map(|pane| TemplatePane {
            command: pane.command.unwrap_or_default().trim().to_string(),
        })
        .collect::<Vec<_>>();

    validate_layout(&id, layout, &panes)?;

    let focus_pane = raw.focus_pane.unwrap_or(0);
    if focus_pane >= panes.len() {
        return Err(format!("Skipped template {id}: focus_pane is out of range"));
    }

    Ok(SessionTemplate {
        id,
        name,
        description,
        layout,
        panes,
        focus_pane,
    })
}

fn required_field(value: Option<String>, field: &str) -> Result<String, String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("Skipped template: missing {field}"))
}

fn validate_layout(id: &str, layout: TemplateLayout, panes: &[TemplatePane]) -> Result<(), String> {
    match layout {
        TemplateLayout::Single if panes.len() != 1 => Err(format!(
            "Skipped template {id}: single layout requires exactly one pane"
        )),
        TemplateLayout::Columns | TemplateLayout::Rows if panes.len() < 2 => Err(format!(
            "Skipped template {id}: split layouts require at least two panes"
        )),
        _ => Ok(()),
    }
}

fn is_valid_template_id(id: &str) -> bool {
    id.chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_skipped_with_warning(catalog: TemplateCatalog, warning: &str) {
        assert_eq!(catalog.templates.len(), 5);
        assert_eq!(catalog.warnings.len(), 1);
        assert!(catalog.warnings[0].contains(warning));
    }

    #[test]
    fn builtins_are_always_available_in_order() {
        let catalog = TemplateCatalog::builtins();
        let ids = catalog
            .templates
            .iter()
            .map(|template| template.id.as_str())
            .collect::<Vec<_>>();

        assert_eq!(
            ids,
            [
                "terminal",
                "two-columns",
                "two-rows",
                "nvim-codex",
                "nvim-gemini"
            ]
        );
        assert!(catalog.warnings.is_empty());
    }

    #[test]
    fn parses_user_template_after_builtins() {
        let catalog = TemplateCatalog::from_config_str(
            r#"
[[templates]]
id = "editor-agent"
name = "Editor + Agent"
description = "nvim on the left, codex on the right"
layout = "columns"
focus_pane = 1
[[templates.panes]]
command = "nvim"
[[templates.panes]]
command = "codex"
"#,
        );

        let template = catalog.templates.last().unwrap();
        assert_eq!(template.id, "editor-agent");
        assert_eq!(template.layout, TemplateLayout::Columns);
        assert_eq!(template.focus_pane, 1);
        assert_eq!(template.pane_summary(), "nvim | codex");
        assert!(catalog.warnings.is_empty());
    }

    #[test]
    fn skips_duplicate_user_template_ids() {
        let catalog = TemplateCatalog::from_config_str(
            r#"
[[templates]]
id = "terminal"
name = "Duplicate"
layout = "single"
[[templates.panes]]
command = ""
"#,
        );

        assert_skipped_with_warning(catalog, "duplicate template id");
    }

    #[test]
    fn skips_invalid_template_ids() {
        let catalog = TemplateCatalog::from_config_str(
            r#"
[[templates]]
id = "bad id"
name = "Bad"
layout = "single"
[[templates.panes]]
command = ""
"#,
        );

        assert_skipped_with_warning(catalog, "letters, digits");
    }

    #[test]
    fn validates_layout_pane_counts() {
        let catalog = TemplateCatalog::from_config_str(
            r#"
[[templates]]
id = "too-few"
name = "Too Few"
layout = "columns"
[[templates.panes]]
command = "nvim"
"#,
        );

        assert_skipped_with_warning(catalog, "at least two panes");
    }

    #[test]
    fn treats_empty_commands_as_shell_panes() {
        let catalog = TemplateCatalog::from_config_str(
            r#"
[[templates]]
id = "shells"
name = "Shells"
layout = "rows"
[[templates.panes]]
command = ""
[[templates.panes]]
command = "   "
"#,
        );

        let template = catalog.templates.last().unwrap();
        assert_eq!(template.pane_summary(), "shell / shell");
    }

    #[test]
    fn rejects_out_of_range_focus_pane() {
        let catalog = TemplateCatalog::from_config_str(
            r#"
[[templates]]
id = "bad-focus"
name = "Bad Focus"
layout = "rows"
focus_pane = 3
[[templates.panes]]
command = ""
[[templates.panes]]
command = ""
"#,
        );

        assert_skipped_with_warning(catalog, "focus_pane is out of range");
    }

    #[test]
    fn invalid_config_keeps_builtins() {
        let catalog = TemplateCatalog::from_config_str("not = [valid");

        assert_eq!(catalog.templates.len(), 5);
        assert_eq!(catalog.warnings.len(), 1);
    }

    #[test]
    fn builds_user_config_path_from_environment_values() {
        assert_eq!(
            user_templates_path_from_env(Some("/tmp/config"), Some("/home/user")).unwrap(),
            PathBuf::from("/tmp/config/trex/templates.toml")
        );

        assert_eq!(
            user_templates_path_from_env(None, Some("/home/user")).unwrap(),
            PathBuf::from("/home/user/.config/trex/templates.toml")
        );

        assert!(user_templates_path_from_env(None, None).is_none());
    }
}
