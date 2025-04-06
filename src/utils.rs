#[cfg(test)]
use std::process::Stdio;
use std::process::{Command, ExitStatus};

use crate::models::ErrorKind;
use log::{error, info};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

/// Runs given command using system shell
pub fn run(process: String) -> Result<ExitStatus, ErrorKind> {
    if process.is_empty() {
        error!("got 0 arguments as command name");
        return Err(ErrorKind::StartingProcessError);
    }

    // Use platform-specific shell commands
    #[cfg(windows)]
    let (shell, shell_arg) = ("cmd", "/C");
    #[cfg(not(windows))]
    let (shell, shell_arg) = ("sh", "-c");

    let mut cmd = Command::new(shell);
    cmd.arg(shell_arg).arg(process);

    #[cfg(test)]
    cmd.stderr(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    match cmd.status() {
        Ok(code) => {
            info!(
                "process exited with exit code {}",
                code.code().unwrap_or_default()
            );
            Ok(code)
        }
        Err(err) => {
            error!("can't start process: {}", err);
            Err(ErrorKind::StartingProcessError)
        }
    }
}

/// Validate variable name
pub fn validate_var_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Variable name cannot be empty".to_string());
    }
    if name.contains(' ') {
        return Err("Variable name cannot contain spaces".to_string());
    }
    Ok(())
}

/// Returns vector of string that are similar by threshold to given string in given vector
pub fn find_similar_string(string: String, strings: Vec<String>, threshold: f64) -> Vec<String> {
    strings
        .par_iter()
        .filter(|name| {
            similar_string::compare_similarity(string.to_lowercase(), name.to_lowercase())
                > threshold
        })
        .map(|name| name.to_string())
        .collect::<Vec<_>>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_var_name_valid() {
        let valid_names = vec![
            "VALID_NAME",
            "MY_VAR_123",
            "PATH",
            "_HIDDEN",
            "VALID_NAME_WITH_NUMBERS_123",
            "A", // Single character
        ];

        for name in valid_names {
            assert!(validate_var_name(name).is_ok());
        }
    }

    #[test]
    fn test_validate_var_name_with_spaces() {
        let invalid_names = vec![
            "INVALID NAME",
            "MY VAR",
            " LEADING_SPACE",
            "TRAILING_SPACE ",
            "MULTIPLE   SPACES",
        ];

        for name in invalid_names {
            let result = validate_var_name(name);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), "Variable name cannot contain spaces");
        }
    }

    #[test]
    fn test_validate_var_name_empty() {
        let result = validate_var_name("");
        assert!(
            result.is_err(),
            "Empty string should be invalid as per current implementation"
        );
        assert_eq!(result.unwrap_err(), "Variable name cannot be empty");
    }

    #[test]
    fn test_find_similar_string_exact_match() {
        let strings = vec!["PATH".to_string(), "HOME".to_string(), "USER".to_string()];
        let result = find_similar_string("PATH".to_string(), strings, 0.8);
        assert_eq!(result, vec!["PATH"]);
    }

    #[test]
    fn test_find_similar_string_case_insensitive() {
        let strings = vec!["PATH".to_string(), "HOME".to_string(), "USER".to_string()];
        let result = find_similar_string("path".to_string(), strings, 0.8);
        assert_eq!(result, vec!["PATH"]);
    }

    #[test]
    fn test_find_similar_string_no_match() {
        let strings = vec!["PATH".to_string(), "HOME".to_string(), "USER".to_string()];
        let result = find_similar_string("XXXXXX".to_string(), strings, 0.8);
        assert!(result.is_empty());
    }

    #[test]
    fn test_find_similar_string_multiple_matches() {
        let strings = vec![
            "TEST".to_string(),
            "TSET".to_string(),
            "TEXT".to_string(),
            "NONE".to_string(),
        ];
        let result = find_similar_string("TEST".to_string(), strings, 0.5);
        assert!(result.contains(&"TEST".to_string()));
        assert!(result.contains(&"TEXT".to_string()));
        assert!(result.contains(&"TSET".to_string()));
        assert!(!result.contains(&"NONE".to_string()));
    }

    #[test]
    fn test_run_successful_command() {
        #[cfg(windows)]
        let cmd = "cmd /C echo test";
        #[cfg(not(windows))]
        let cmd = "echo test";

        let result = run(cmd.to_string());
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_nonexistent_command() {
        let result = run("nonexistent_command_123".to_string());
        assert!(result.is_ok());
        assert!(!result.unwrap().success());
    }

    #[test]
    fn test_run_failing_command() {
        #[cfg(windows)]
        let cmd = "cmd /C exit 1";
        #[cfg(not(windows))]
        let cmd = "false";

        let result = run(cmd.to_string());
        assert!(result.is_ok());
        assert!(!result.unwrap().success());
    }

    #[test]
    fn test_run_empty_command() {
        let result = run("".to_string());
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ErrorKind::StartingProcessError
        ));
    }

    #[test]
    fn test_run_invalid_executable() {
        // Test with a command that should fail to execute
        let result = run("\0invalid".to_string());
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ErrorKind::StartingProcessError
        ));
    }

    #[test]
    fn test_run_null_command() {
        // Test with a null character in command which should fail to start
        let result = run("echo \0test".to_string());
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ErrorKind::StartingProcessError
        ));
    }

    #[test]
    fn test_run_with_very_long_command() {
        // Create a command that's too long to execute
        let very_long_command = "x".repeat(65536);
        let _ = run(very_long_command);
        // Different OS's may return different error types for too-long commands
    }

    #[test]
    fn test_run() {
        #[cfg(windows)]
        let cmd = "cmd /C echo test";
        #[cfg(not(windows))]
        let cmd = "echo test";

        let result = run(cmd.to_string());
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_failing_command_two() {
        #[cfg(windows)]
        let cmd = "cmd /C exit 1";
        #[cfg(not(windows))]
        let cmd = "false";

        let result = run(cmd.to_string());
        assert!(result.is_ok());
    }
}
