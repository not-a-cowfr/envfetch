use colored::Colorize;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::process;
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
    eprintln!("{} {}", "error:".red(), text);
}

/// Print info about warning
pub fn warning(text: &str, exit_on_warning: bool) {
    eprintln!("{} {}", "warning:".yellow(), text);
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
}
