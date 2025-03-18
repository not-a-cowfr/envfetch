use std::{fs, io, path::PathBuf};

use dirs::config_dir;

use crate::models::{Config, ConfigParsingError};

/// Get path to config file
fn get_config_dir() -> PathBuf {
    config_dir().unwrap_or(PathBuf::new())
}

/// Read config file
pub fn read_config() -> Result<Config, ConfigParsingError> {
    let path = get_config_dir().join("envfetch.toml");
    if !path.exists() {
        return Err(ConfigParsingError::FileDoesntExists);
    }

    let content = fs::read_to_string(path).map_err(|err | ConfigParsingError::FSError(err.to_string()))?;

    toml::from_str::<Config>(&content).map_err(|err| ConfigParsingError::ParsingError(err.to_string()))
}

/// Initialize config file
pub fn init_config() -> io::Result<()> {
    let default = include_str!("../assets/default_config.toml");

    let file = get_config_dir().join("envfetch.toml");

    fs::write(&file, default).map(|_| eprintln!("Successfully initialized config at {}", file.display()))
}

#[cfg(test)]
mod tests {
    use crate::config::get_config_dir;
    use dirs::config_dir;
    use std::path::PathBuf;

    #[test]
    fn test_get_config_dir() {
        assert_eq!(get_config_dir(), config_dir().unwrap_or(PathBuf::new()));
    }
}
