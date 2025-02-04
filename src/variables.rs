use colored::Colorize;
use std::env;

use crate::{models::ErrorKind, utils::*};

/// Print all environment variables
pub fn print_env() {
    for (key, value) in env::vars() {
        println!("{} = \"{}\"", key.blue(), value);
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
        return run(process, false);
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
        print_env();
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

    #[test]
    fn test_print_env_format() {
        // Set up test environment variables
        env::set_var("TEST_VAR_1", "value1");
        env::set_var("TEST_VAR_2", "value2");

        print_env();
        
        // Clean up
        env::remove_var("TEST_VAR_1");
        env::remove_var("TEST_VAR_2");
    }

    #[test]
    fn test_print_env_empty_value() {
        env::set_var("TEST_EMPTY", "");

        print_env();

        env::remove_var("TEST_EMPTY");
    }

    #[test]
    fn test_print_env_special_characters() {
        env::set_var("TEST_SPECIAL", "value with spaces and $#@!");

        print_env();

        env::remove_var("TEST_SPECIAL");
    }

    #[test]
    fn test_set_variable_global() {
        let result = set_variable("TEST_GLOBAL_VAR", "test_value", true, None);
        match result {
            Ok(_) => {
                assert_eq!(env::var("TEST_GLOBAL_VAR").unwrap(), "test_value");
                delete_variable("TEST_GLOBAL_VAR".to_string(), true).unwrap();
            },
            Err(ErrorKind::CannotSetVariableGlobally(_)) => {
                // Test passes if we get permission error on non-admin run
            },
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn test_set_variable_global_with_process() {
        #[cfg(windows)]
        let cmd = "cmd /C echo test";
        #[cfg(not(windows))]
        let cmd = "echo test";

        let result = set_variable("TEST_GLOBAL_PROC", "test_value", true, Some(cmd.to_string()));
        match result {
            Ok(_) => {
                assert_eq!(env::var("TEST_GLOBAL_PROC").unwrap(), "test_value");
                delete_variable("TEST_GLOBAL_PROC".to_string(), true).unwrap();
            },
            Err(ErrorKind::CannotSetVariableGlobally(_)) => {
                // Test passes if we get permission error on non-admin run
            },
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn test_delete_variable_global() {
        // First try to set a global variable
        let set_result = set_variable("TEST_GLOBAL_DELETE", "test_value", true, None);
        
        // Only test deletion if we could set the variable (i.e., we have admin rights)
        if set_result.is_ok() {
            let result = delete_variable("TEST_GLOBAL_DELETE".to_string(), true);
            assert!(result.is_ok());
            assert!(env::var("TEST_GLOBAL_DELETE").is_err());
        }
    }

    #[test]
    fn test_delete_nonexistent_variable_global() {
        let result = delete_variable("NONEXISTENT_GLOBAL_VAR".to_string(), true);
        match result {
            Ok(_) => {},
            Err(ErrorKind::CannotDeleteVariableGlobally(_)) => {
                // Test passes if we get permission error on non-admin run
            },
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }
}
