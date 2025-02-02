use colored::Colorize;
use std::env;

use crate::{models::ErrorKind, utils::*};

/// Print all environment variables
pub fn print_env(writer: &mut dyn std::io::Write) {
    for (key, value) in env::vars() {
        writeln!(writer, "{} = \"{}\"", key.blue(), value).expect("can't write to buffer");
    }
}

/// Set variable with given key and value
pub fn set_variable(
    key: &str,
    value: &str,
    global: bool,
    process: Option<String>,
) -> Result<(), ErrorKind> {
    if global {
        if let Err(err) = globalenv::set_var(key, value) {
            return Err(ErrorKind::CannotSetVariableGlobally(err.to_string()));
        }
    } else {
        unsafe { env::set_var(key, value) };
    }

    if let Some(process) = process {
        return run(process);
    }
    Ok(())
}

/// Delete variable with given name
pub fn delete_variable(name: String, global: bool) -> Result<(), ErrorKind> {
    if global {
        if let Err(err) = globalenv::unset_var(&name) {
            return Err(ErrorKind::CannotDeleteVariableGlobally(err.to_string()));
        }
    } else {
        unsafe { env::remove_var(&name) };
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use std::env;

    #[test]
    fn test_set_variable_simple() {
        let result = set_variable("TEST_VAR", "test_value", false, None);
        assert!(result.is_ok());
        assert_eq!(env::var("TEST_VAR").unwrap(), "test_value");
        env::remove_var("TEST_VAR");
    }

    #[test]
    fn test_set_variable_with_process() {
        #[cfg(windows)]
        let cmd = "cmd /C echo test";
        #[cfg(not(windows))]
        let cmd = "echo test";

        let result = set_variable("TEST_PROC_VAR", "test_value", false, Some(cmd.to_string()));
        assert!(result.is_ok());
        assert_eq!(env::var("TEST_PROC_VAR").unwrap(), "test_value");
        env::remove_var("TEST_PROC_VAR");
    }

    #[test]
    fn test_print_env() {
        env::set_var("TEST_PRINT_VAR", "test_value");
        let mut output = Cursor::new(Vec::new());
        print_env(&mut output);
        env::remove_var("TEST_PRINT_VAR");
    }

    #[test]
    fn test_set_variable_invalid_process() {
        let result = set_variable(
            "TEST_INVALID_PROC", 
            "test_value", 
            false, 
            Some("nonexistent_command".to_string())
        );
        
        // Check that the operation failed
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ErrorKind::ProcessFailed));
        
        // Check that the variable is still set despite process failure
        assert_eq!(env::var("TEST_INVALID_PROC").unwrap(), "test_value");
        
        // Cleanup
        env::remove_var("TEST_INVALID_PROC");
    }

    #[test]
    fn test_delete_variable() {
        env::set_var("TEST_DELETE_VAR", "test_value");
        let result = delete_variable("TEST_DELETE_VAR".to_string(), false);
        assert!(result.is_ok());
        assert!(env::var("TEST_DELETE_VAR").is_err());
    }

    #[test]
    fn test_set_variable_empty_value() {
        let result = set_variable("TEST_EMPTY_VAR", "", false, None);
        assert!(result.is_ok());
        assert_eq!(env::var("TEST_EMPTY_VAR").unwrap(), "");
        env::remove_var("TEST_EMPTY_VAR");
    }
}
