use std::{
    fs,
    io::{self, Write},
    path::PathBuf,
};

use dirs::config_dir;

use crate::models::{Config, ConfigParsingError};

/// Get path to config directory
fn get_config_dir() -> PathBuf {
    config_dir().unwrap_or_default()
}

/// Get path to config file
pub fn get_config_file_path() -> PathBuf {
    get_config_dir().join("envfetch.toml")
}

/// Read config file
pub fn read_config_from_file(path: PathBuf) -> Result<Config, ConfigParsingError> {
    if !path.exists() {
        return Err(ConfigParsingError::FileDoesntExists);
    }

    let content =
        fs::read_to_string(path).map_err(|err| ConfigParsingError::FSError(err.to_string()))?;

    read_config(content)
}

/// Read config file
fn read_config(content: String) -> Result<Config, ConfigParsingError> {
    toml::from_str::<Config>(&content)
        .map_err(|err| ConfigParsingError::ParsingError(err.to_string()))
}

/// Initialize config file
pub fn init_config<W: Write>(path: PathBuf, mut buffer: W) -> io::Result<()> {
    let default = default_config();
    fs::write(&path, default)?;
    writeln!(
        buffer,
        "Successfully initialized config at {}",
        path.display()
    )?;
    Ok(())
}

/// Get default config ile content
fn default_config() -> &'static str {
    include_str!("../assets/default_config.toml")
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::prelude::*;
    use dirs::config_dir;

    // Add new struct for testing write failures
    struct FailingWriter;

    impl Write for FailingWriter {
        fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
            Err(io::Error::new(io::ErrorKind::Other, "Mock write error"))
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_get_config_dir() {
        assert_eq!(get_config_dir(), config_dir().unwrap_or_default());
    }

    #[test]
    fn test_get_config_file() {
        assert_eq!(
            get_config_file_path(),
            config_dir().unwrap_or_default().join("envfetch.toml")
        );
    }

    #[test]
    fn test_default_config() {
        assert_eq!(
            default_config(),
            include_str!("../assets/default_config.toml")
        );
    }

    #[test]
    fn test_read_config_default() {
        let result = read_config(default_config().to_owned()).unwrap();
        assert_eq!(result, Config { print_format: None })
    }

    #[test]
    fn test_read_config_from_existent_file() {
        let file = assert_fs::NamedTempFile::new("envfetch.toml").unwrap();
        file.write_str(default_config()).unwrap();
        let result = read_config_from_file(file.path().to_path_buf()).unwrap();
        assert_eq!(result, Config { print_format: None })
    }

    #[test]
    fn test_read_config_from_invalid_file() {
        let file = assert_fs::NamedTempFile::new("envfetch.toml").unwrap();
        // Just print anything wrong to file
        file.write_str("aegkbiv wlecn k").unwrap();
        match read_config_from_file(file.path().to_path_buf()) {
            Err(ConfigParsingError::ParsingError(_)) => (),
            _ => panic!("Should crash with ParsingError"),
        }
    }

    #[test]
    fn test_read_config_from_unexistent_file() {
        let file = assert_fs::NamedTempFile::new("envfetch.toml").unwrap();
        let path = file.path().to_path_buf();
        file.close().unwrap();
        let result = read_config_from_file(path);
        assert_eq!(result, Err(ConfigParsingError::FileDoesntExists))
    }

    #[test]
    fn test_init_config() {
        let file = assert_fs::NamedTempFile::new("envfetch.toml").unwrap();
        let mut buffer = vec![];
        init_config(file.path().to_path_buf(), &mut buffer).unwrap();
        assert!(String::from_utf8(buffer).unwrap().contains(&format!(
            "Successfully initialized config at {}",
            file.display()
        )));
    }

    // Add new test for successful buffer writing
    #[test]
    fn test_init_config_buffer_write() -> io::Result<()> {
        let file = assert_fs::NamedTempFile::new("envfetch.toml").unwrap();
        let mut buffer = Vec::new();
        init_config(file.path().to_path_buf(), &mut buffer)?;

        let written = String::from_utf8(buffer).unwrap();
        assert!(written.contains("Successfully initialized"));
        assert!(written.contains(&file.path().display().to_string()));
        Ok(())
    }

    #[test]
    fn test_init_config_buffer_write_failure() {
        let file = assert_fs::NamedTempFile::new("envfetch.toml").unwrap();
        let mut failing_writer = FailingWriter;

        let result = init_config(file.path().to_path_buf(), &mut failing_writer);
        assert!(result.is_err()); // Now we expect an error since the buffer write fails
        assert!(file.exists()); // File should still be created even though buffer write failed
    }

    #[test]
    fn test_init_config_file_write_failure() -> io::Result<()> {
        let non_existent_dir = PathBuf::from("/non/existent/dir/envfetch.toml");
        let mut buffer = Vec::new();
        let result = init_config(non_existent_dir, &mut buffer);
        assert!(result.is_err()); // This should fail on fs::write
        Ok(())
    }
}
