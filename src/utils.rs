use colored::Colorize;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::process;
use subprocess::Exec;
use std::io::stderr;

/// Runs given command using system shell
pub fn run(process: String) {
    let result = Exec::shell(process).join().unwrap_or_else(|_| {
        error("can't start process", &mut stderr());
        // Exit with non-zero exit code if we can't start process
        process::exit(1);
    });

    // Exit with non-zero exit code if process did not successful
    if !result.success() {
        process::exit(1);
    }
}

/// Validate variable name
pub fn validate_var_name(name: &str) -> Result<(), String> {
    if name.contains(' ') {
        return Err("Variable name cannot contain spaces".into());
    }
    Ok(())
}

/// Print info about error
pub fn error(text: &str, writer: &mut dyn std::io::Write) {
    writeln!(writer, "{} {}", "error:".red(), text).expect("can't write to buffer");
}

/// Print info about warning
pub fn warning(text: &str, exit_on_warning: bool, writer: &mut dyn std::io::Write) {
    writeln!(writer, "{} {}", "warning:".yellow(), text).expect("can't write to buffer");
    if exit_on_warning {
        process::exit(1);
    }
}

/// Returns vector of string that are similar by threshold to given string in given vector
pub fn find_similar_string(string: String, strings: Vec<String>, threshold:f64) -> Vec<String> {
    strings.par_iter()
        .filter(|name| {
            similar_string::compare_similarity(
                string.to_lowercase(),
                name.to_lowercase(),
            ) > threshold
        })
        .map(|name| name.to_string())
        .collect::<Vec<_>>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_output() {
        let mut output = Vec::new();
        error("test error message", &mut output);
        
        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("error:"));
        assert!(output_str.contains("test error message"));
    }

    #[test]
    fn test_warning_output() {
        let mut output = Vec::new();
        warning("test warning message", false, &mut output);
        
        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("warning:"));
        assert!(output_str.contains("test warning message"));
    }

    #[test]
    fn test_error_empty_message() {
        let mut output = Vec::new();
        error("", &mut output);
        
        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("error:"));
    }

    #[test]
    fn test_warning_empty_message() {
        let mut output = Vec::new();
        warning("", false, &mut output);
        
        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("warning:"));
    }

    // TODO: Add tests for warning function's exit_on_warning behavior

    #[test]
    fn test_validate_var_name_valid() {
        let valid_names = vec![
            "VALID_NAME",
            "MY_VAR_123",
            "PATH",
            "_HIDDEN",
            "VALID_NAME_WITH_NUMBERS_123",
            "A",  // Single character
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
        assert!(result.is_ok(), "Empty string should be valid as per current implementation");
    }

    #[test]
    fn test_find_similar_string_exact_match() {
        let strings = vec![
            "PATH".to_string(),
            "HOME".to_string(),
            "USER".to_string()
        ];
        let result = find_similar_string("PATH".to_string(), strings, 0.8);
        assert_eq!(result, vec!["PATH"]);
    }

    #[test]
    fn test_find_similar_string_case_insensitive() {
        let strings = vec![
            "PATH".to_string(),
            "HOME".to_string(),
            "USER".to_string()
        ];
        let result = find_similar_string("path".to_string(), strings, 0.8);
        assert_eq!(result, vec!["PATH"]);
    }

    #[test]
    fn test_find_similar_string_no_match() {
        let strings = vec![
            "PATH".to_string(),
            "HOME".to_string(),
            "USER".to_string()
        ];
        let result = find_similar_string("XXXXXX".to_string(), strings, 0.8);
        assert!(result.is_empty());
    }

    #[test]
    fn test_find_similar_string_multiple_matches() {
        let strings = vec![
            "TEST".to_string(),
            "TSET".to_string(),
            "TEXT".to_string(),
            "NONE".to_string()
        ];
        let result = find_similar_string("TEST".to_string(), strings, 0.5);
        assert!(result.contains(&"TEST".to_string()));
        assert!(result.contains(&"TEXT".to_string()));
        assert!(result.contains(&"TSET".to_string()));
        assert!(!result.contains(&"NONE".to_string()));
    }
}
