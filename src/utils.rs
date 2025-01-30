use colored::Colorize;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{io::stderr, process};
use subprocess::Exec;

/// Runs given command using system shell
pub fn run(process: String) {
    let result = Exec::shell(process).join().unwrap_or_else(|_| {
        error("can't start process");
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
pub fn error(text: &str) {
    print_error(text, &mut stderr());
}

/// Print info about warning
pub fn warning(text: &str, exit_on_warning: bool) {
    print_warning(text, &mut stderr());
    if exit_on_warning {
        process::exit(1);
    }
}

/// Print warning to buffer
fn print_warning(text: &str, writer: &mut dyn std::io::Write) {
    write!(writer, "{} {}", "warning:".yellow(), text).expect("can't write to buffer");
}

/// Print error to buffer 
fn print_error(text: &str, writer: &mut dyn std::io::Write) {
    write!(writer, "{} {}", "error:".red(), text).expect("can't write to buffer");
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
    fn test_validate_var_name_valid() {
        let result = validate_var_name("VALID_NAME");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_var_name_with_space() {
        let result = validate_var_name("INVALID NAME");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Variable name cannot contain spaces");
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

    #[test]
    fn test_print_warning() {
        let mut buffer = Vec::new();
        print_warning("test warning message", &mut buffer);
        let result = String::from_utf8(buffer).unwrap();
        // Note: we can't test the exact color codes as they may vary by environment,
        // but we can test the basic message structure
        assert!(result.contains("warning:"));
        assert!(result.contains("test warning message"));
    }

    #[test]
    fn test_print_error() {
        let mut buffer = Vec::new();
        print_error("test error message", &mut buffer);
        let result = String::from_utf8(buffer).unwrap();
        assert!(result.contains("error:"));
        assert!(result.contains("test error message"));
    }

    #[test]
    fn test_print_warning_empty_message() {
        let mut buffer = Vec::new();
        print_warning("", &mut buffer);
        let result = String::from_utf8(buffer).unwrap();
        assert!(result.contains("warning:"));
    }

    #[test]
    fn test_print_error_empty_message() {
        let mut buffer = Vec::new();
        print_error("", &mut buffer);
        let result = String::from_utf8(buffer).unwrap();
        assert!(result.contains("error:"));
    }

    #[test]
    fn test_print_warning_special_characters() {
        let mut buffer = Vec::new();
        print_warning("test @#$%^&* message", &mut buffer);
        let result = String::from_utf8(buffer).unwrap();
        assert!(result.contains("warning:"));
        assert!(result.contains("test @#$%^&* message"));
    }

    #[test]
    fn test_print_error_special_characters() {
        let mut buffer = Vec::new();
        print_error("test @#$%^&* message", &mut buffer);
        let result = String::from_utf8(buffer).unwrap();
        assert!(result.contains("error:"));
        assert!(result.contains("test @#$%^&* message"));
    }
}
