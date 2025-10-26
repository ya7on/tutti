use tutti_types::Project;

mod adapter;
mod raw;

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
        "toml" => parse_toml(text, path),
        _ => Err(ConfigError::Validation("unknown config extension".into())),
    }
}

/// Parse a project configuration from a string.
///
/// # Errors
///
/// Returns a `ConfigError` if the configuration string cannot be parsed.
#[cfg(feature = "toml")]
pub fn parse_toml(config: &str, path: &std::path::Path) -> Result<Project, ConfigError> {
    let raw_project = toml::from_str::<raw::RawProject>(config)?;
    raw_project.to_project(path)
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, path::PathBuf};

    use tutti_types::Restart;

    use super::*;

    #[test]
    fn parse_toml_ok() {
        let txt = r#"
            version = 100500

            [services.api]
            cmd = ["cargo","run","--bin","api"]
            cwd = "/home/user"
            env = { RUST_LOG = "info" }
            deps = ["db"]
            restart = "always"

            [services.db]
            cmd = ["postgres","-D",".pg"]
            restart = "never"
        "#;
        let p = parse_toml(txt, std::path::Path::new("config.toml")).unwrap();
        assert!(p.services.contains_key("api"));
        assert_eq!(p.version, 100500);
        assert_eq!(p.services["api"].cmd, vec!["cargo", "run", "--bin", "api"]);
        assert_eq!(p.services["api"].cwd, Some(PathBuf::from("/home/user")));
        assert_eq!(
            p.services["api"].env,
            Some(HashMap::from_iter(vec![(
                "RUST_LOG".to_string(),
                "info".to_string()
            )]))
        );
        assert_eq!(p.services["api"].deps, vec!["db"]);
        assert_eq!(p.services["api"].restart, Restart::Always);
        assert_eq!(p.services["db"].cmd, vec!["postgres", "-D", ".pg"]);
        assert_eq!(p.services["db"].cwd, None);
        assert_eq!(p.services["db"].env, None);
        assert!(p.services["db"].deps.is_empty());
        assert_eq!(p.services["db"].restart, Restart::Never);
    }

    #[test]
    fn parse_auto_ok() {
        let txt = r#"
            [services.api]
            cmd = ["cargo","run","--bin","api"]

            [services.db]
            cmd = ["postgres","-D",".pg"]
        "#;
        let p = parse_auto(txt, std::path::Path::new("config.toml")).unwrap();
        assert!(p.services.contains_key("api"));
        assert_eq!(p.services["api"].cmd, vec!["cargo", "run", "--bin", "api"]);
        assert_eq!(p.version, 1);
    }

    #[test]
    fn parse_auto_unknown_format() {
        let txt = r#"
            UnknownFormat
        "#;
        let result = parse_auto(txt, std::path::Path::new("config.unknown"));
        assert!(result.is_err());
    }

    #[test]
    fn load_from_path_ok() {
        let path = PathBuf::from("../../tests/assets/correct_config.toml");
        let p = load_from_path(&path).unwrap();
        assert!(p.services.contains_key("service"));
    }

    #[test]
    fn load_from_path_unknown_format() {
        let path = PathBuf::from("../../tests/assets/unknown_format.toml");
        let result = load_from_path(&path);
        assert!(result.is_err());
    }
}
