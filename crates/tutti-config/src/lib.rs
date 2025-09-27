mod model;
mod raw;

pub use model::Project;

/// Error type for configuration parsing.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[cfg(feature = "toml")]
    #[error("toml parse error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("validation error(s): {0}")]
    Validation(String),
}

/// Load a project configuration from a file path.
///
/// # Errors
///
/// Returns a `ConfigError` if the configuration file cannot be read or parsed.
pub fn load_from_path(path: &std::path::Path) -> Result<Project, ConfigError> {
    let text = std::fs::read_to_string(path)?;
    parse_auto(&text, path)
}

/// Parse a project configuration from a string.
///
/// # Errors
///
/// Returns a `ConfigError` if the configuration string cannot be parsed.
pub fn parse_auto(text: &str, path: &std::path::Path) -> Result<Project, ConfigError> {
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
    match ext {
        #[cfg(feature = "toml")]
        "toml" => parse_toml(text),
        _ => Err(ConfigError::Validation("unknown config extension".into())),
    }
}

/// Parse a project configuration from a string.
///
/// # Errors
///
/// Returns a `ConfigError` if the configuration string cannot be parsed.
#[cfg(feature = "toml")]
pub fn parse_toml(config: &str) -> Result<Project, ConfigError> {
    let raw_project = toml::from_str::<raw::RawProject>(config)?;
    raw_project.try_into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_toml_ok() {
        let txt = r#"
            version = 1
            [services.api]
            cmd = ["cargo","run","--bin","api"]

            [services.db]
            cmd = ["postgres","-D",".pg"]
        "#;
        let p = parse_toml(txt).unwrap();
        assert!(p.services.contains_key("api"));
        assert_eq!(p.services["api"].cmd, vec!["cargo", "run", "--bin", "api"]);
        assert_eq!(p.version, 1);
    }
}
