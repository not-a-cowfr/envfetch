use std::env;
use colored::Colorize;

use crate::utils::*;

/// Print all environment variables
pub fn print_env(writer: &mut dyn std::io::Write) {
    for (key, value) in env::vars() {
        writeln!(writer, "{} = \"{}\"", key.blue(), value).expect("can't write to buffer");
    }
}

/// Set variable with given key and value
pub fn set_variable(key: &str, value: &str, global: bool, process: Option<String>) -> Result<(), String> {
    if global {
        if let Err(err) = globalenv::set_var(key, value) {
            return Err(
                format!(
                    "can't globally set variable: {} (do you have the required permissions?)",
                    err
                )
            );
        }
    } else {
        unsafe { env::set_var(key, value) };
    }

    if let Some(process) = process {
        run(process);
    }
    Ok(())
}

/// Delete variable with given name
pub fn delete_variable(name: String, global: bool) -> Result<(), String> {
    if global {
        if let Err(err) = globalenv::unset_var(&name) {
            return Err(
                format!(
                    "can't globally delete variable: {} (do you have the required permissions?)",
                    err
                )
            );
        }
    } else {
        unsafe { env::remove_var(&name) };
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_variable_local() {
        let key = "TEST_LOCAL_VAR";
        let value = "test_value";
        let _ = set_variable(key, value, false, None);
        assert_eq!(env::var(key).unwrap(), value);
    }

    #[test]
    fn test_set_variable_global_error() {
        let key = "TEST_GLOBAL_VAR";
        let value = "test_value";
        // This should fail without admin privileges and trigger the error path
        let _ = set_variable(key, value, true, None);
        // We can't assert the global state as it depends on permissions
    }

    #[test]
    fn test_set_variable_with_process() {
        let key = "TEST_PROCESS_VAR";
        let value = "test_value";
        // Using echo as a test process - it should not fail
        let _ = set_variable(key, value, false, Some("echo test".to_string()));
        assert_eq!(env::var(key).unwrap(), value);
    }

    #[test]
    fn test_delete_variable_local() {
        let key = "TEST_DELETE_LOCAL_VAR";
        env::set_var(key, "test_value");
        assert!(env::var(key).is_ok());
        
        let result = delete_variable(key.to_string(), false);
        assert!(result.is_ok());
        assert!(env::var(key).is_err());
    }

    #[test]
    fn test_delete_variable_nonexistent() {
        let key = "NONEXISTENT_VAR";
        let result = delete_variable(key.to_string(), false);
        assert!(result.is_ok()); // Should succeed even if variable doesn't exist
    }

    #[test]
    fn test_delete_variable_global_error() {
        let key = "TEST_DELETE_GLOBAL_VAR";
        // This should fail without admin privileges
        let _ = delete_variable(key.to_string(), true);
        // We can't assert the global state as it depends on permissions
    }

    #[test]
    fn test_print_env() {
        let mut output = Vec::new();
        env::set_var("TEST_PRINT_VAR", "test_value");
        
        print_env(&mut output);
        
        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("TEST_PRINT_VAR"));
        assert!(output_str.contains("test_value"));
    }

    #[test]
    fn test_print_env_empty() {
        let mut output = Vec::new();
        let key = "TEST_PRINT_EMPTY";
        env::remove_var(key); // Ensure the variable doesn't exist
        
        print_env(&mut output);
        
        let output_str = String::from_utf8(output).unwrap();
        assert!(!output_str.contains(key));
    }

    #[test]
    fn test_print_env_multiple() {
        let mut output = Vec::new();
        env::set_var("TEST_VAR_1", "value1");
        env::set_var("TEST_VAR_2", "value2");
        
        print_env(&mut output);
        
        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("TEST_VAR_1"));
        assert!(output_str.contains("value1"));
        assert!(output_str.contains("TEST_VAR_2"));
        assert!(output_str.contains("value2"));
        
        // Cleanup
        env::remove_var("TEST_VAR_1");
        env::remove_var("TEST_VAR_2");
    }
}
