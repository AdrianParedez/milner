use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fmt;
use std::path::{Path, PathBuf};

use super::parser::{
    ExecutionPlan, InputSpec, OutputSpec, ParseError, ParsedCommand, parse_execution_line,
};

pub(super) const DEFAULT_PROMPT: &str = "milner> ";

#[derive(Clone, Debug)]
pub(super) enum ConfigSource {
    Default,
    Disabled,
    Path(PathBuf),
}

#[derive(Clone, Debug)]
pub(super) struct ShellConfig {
    pub prompt: PromptConfig,
    pub history: HistoryConfig,
    pub aliases: BTreeMap<String, ParsedCommand>,
}

#[derive(Clone, Debug)]
pub(super) struct PromptConfig {
    pub text: String,
}

#[derive(Clone, Debug, Default)]
pub(super) struct HistoryConfig {
    pub enabled: bool,
    pub path: Option<PathBuf>,
}

#[derive(Debug)]
pub(crate) enum ConfigError {
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    Invalid {
        path: PathBuf,
        line: usize,
        message: String,
    },
}

impl ShellConfig {
    pub fn load(source: ConfigSource) -> Result<Self, ConfigError> {
        let path = match source {
            ConfigSource::Default => {
                let Some(path) = default_config_path() else {
                    return Ok(Self::default());
                };
                if !path.exists() {
                    return Ok(Self::default());
                }
                path
            }
            ConfigSource::Disabled => return Ok(Self::default()),
            ConfigSource::Path(path) => path,
        };

        let input = std::fs::read_to_string(&path).map_err(|source| ConfigError::Read {
            path: path.clone(),
            source,
        })?;
        parse_config(&path, &input)
    }
}

impl Default for ShellConfig {
    fn default() -> Self {
        Self {
            prompt: PromptConfig {
                text: DEFAULT_PROMPT.to_string(),
            },
            history: HistoryConfig::default(),
            aliases: BTreeMap::new(),
        }
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read { path, source } => {
                write!(f, "config `{}` could not be read: {source}", path.display())
            }
            Self::Invalid {
                path,
                line,
                message,
            } => {
                write!(f, "config `{}` line {line}: {message}", path.display())
            }
        }
    }
}

impl std::error::Error for ConfigError {}

enum Section {
    Root,
    Prompt,
    History,
    Aliases,
}

fn parse_config(path: &Path, input: &str) -> Result<ShellConfig, ConfigError> {
    let mut config = ShellConfig::default();
    let mut section = Section::Root;

    for (index, raw_line) in input.lines().enumerate() {
        let line_number = index + 1;
        let line = strip_comment(raw_line).trim();
        if line.is_empty() {
            continue;
        }

        if line.starts_with('[') || line.ends_with(']') {
            section = parse_section(path, line_number, line)?;
            continue;
        }

        let (key, value) = split_key_value(path, line_number, line)?;
        match section {
            Section::Root => {
                return Err(invalid(
                    path,
                    line_number,
                    "settings must be inside a section",
                ));
            }
            Section::Prompt => parse_prompt_key(path, line_number, key, value, &mut config)?,
            Section::History => parse_history_key(path, line_number, key, value, &mut config)?,
            Section::Aliases => parse_alias(path, line_number, key, value, &mut config)?,
        }
    }

    Ok(config)
}

fn parse_section(path: &Path, line: usize, value: &str) -> Result<Section, ConfigError> {
    match value {
        "[prompt]" => Ok(Section::Prompt),
        "[history]" => Ok(Section::History),
        "[aliases]" => Ok(Section::Aliases),
        _ => Err(invalid(path, line, "unknown or malformed section")),
    }
}

fn parse_prompt_key(
    path: &Path,
    line: usize,
    key: &str,
    value: &str,
    config: &mut ShellConfig,
) -> Result<(), ConfigError> {
    match key {
        "text" => {
            let text = parse_text_value(path, line, value)?;
            if text.is_empty() {
                return Err(invalid(path, line, "prompt.text must not be empty"));
            }
            config.prompt.text = text;
            Ok(())
        }
        _ => Err(invalid(path, line, "unknown prompt key")),
    }
}

fn parse_history_key(
    path: &Path,
    line: usize,
    key: &str,
    value: &str,
    config: &mut ShellConfig,
) -> Result<(), ConfigError> {
    match key {
        "enabled" => {
            config.history.enabled = parse_bool(path, line, value)?;
            Ok(())
        }
        "path" => {
            config.history.path = Some(PathBuf::from(parse_text_value(path, line, value)?));
            Ok(())
        }
        _ => Err(invalid(path, line, "unknown history key")),
    }
}

fn parse_alias(
    path: &Path,
    line: usize,
    key: &str,
    value: &str,
    config: &mut ShellConfig,
) -> Result<(), ConfigError> {
    validate_alias_name(path, line, key)?;
    let plan = parse_execution_line(value).map_err(|err| alias_parse_error(path, line, err))?;
    let command = match plan {
        ExecutionPlan::Command(command)
            if command.stdin == InputSpec::Inherit && command.stdout == OutputSpec::Inherit =>
        {
            command.command
        }
        ExecutionPlan::Command(_) => {
            return Err(invalid(
                path,
                line,
                "alias values must not include redirection",
            ));
        }
        ExecutionPlan::Pipeline { .. } => {
            return Err(invalid(
                path,
                line,
                "alias values must not include pipelines",
            ));
        }
    };
    config.aliases.insert(key.to_string(), command);
    Ok(())
}

fn split_key_value<'a>(
    path: &Path,
    line: usize,
    value: &'a str,
) -> Result<(&'a str, &'a str), ConfigError> {
    let Some((key, value)) = value.split_once('=') else {
        return Err(invalid(path, line, "expected key = value"));
    };

    let key = key.trim();
    if key.is_empty() {
        return Err(invalid(path, line, "key must not be empty"));
    }

    Ok((key, value.trim()))
}

fn parse_text_value(path: &Path, line: usize, value: &str) -> Result<String, ConfigError> {
    if value.is_empty() {
        return Err(invalid(path, line, "value must not be empty"));
    }

    if value.starts_with('"') || value.ends_with('"') {
        if value.len() < 2 || !value.starts_with('"') || !value.ends_with('"') {
            return Err(invalid(path, line, "quoted values must end with a quote"));
        }

        let inner = &value[1..value.len() - 1];
        if inner.contains('"') {
            return Err(invalid(path, line, "quoted values cannot contain quotes"));
        }

        return Ok(inner.to_string());
    }

    Ok(value.to_string())
}

fn parse_bool(path: &Path, line: usize, value: &str) -> Result<bool, ConfigError> {
    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(invalid(path, line, "boolean values must be true or false")),
    }
}

fn validate_alias_name(path: &Path, line: usize, name: &str) -> Result<(), ConfigError> {
    if name.is_empty() {
        return Err(invalid(path, line, "alias name must not be empty"));
    }

    if name
        .chars()
        .any(|ch| ch.is_whitespace() || matches!(ch, '\\' | '/' | ':' | '|' | '<' | '>'))
    {
        return Err(invalid(
            path,
            line,
            "alias name contains unsupported characters",
        ));
    }

    Ok(())
}

fn strip_comment(line: &str) -> &str {
    line.split_once('#').map_or(line, |(before, _)| before)
}

fn default_config_path() -> Option<PathBuf> {
    std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .map(|path| path.join("milner").join("config.toml"))
}

pub(super) fn default_history_path() -> Option<PathBuf> {
    std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .map(|path| path.join("milner").join("history.txt"))
}

fn alias_parse_error(path: &Path, line: usize, err: ParseError) -> ConfigError {
    invalid(path, line, &format!("alias value is invalid: {err}"))
}

fn invalid(path: &Path, line: usize, message: &str) -> ConfigError {
    ConfigError::Invalid {
        path: path.to_path_buf(),
        line,
        message: message.to_string(),
    }
}

pub(super) fn os_string_to_alias_key(value: &OsString) -> Option<&str> {
    value.to_str()
}
