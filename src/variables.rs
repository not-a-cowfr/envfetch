use std::{env, io::Write};

use crate::{models::ErrorKind, utils::*};

/// List of variables
type VariablesList = Vec<(String, String)>;

/// Print all environment variables
pub fn print_env<W: Write>(format: &str, mut buffer: W) {
    for (key, value) in get_variables() {
        let entry = format.replace("{name}", &key).replace("{value}", &value);
        writeln!(buffer, "{}", entry).expect("Failed to write to buffer");
    }
}

/// Get list of environment variables with values
pub fn get_variables() -> VariablesList {
    env::vars().collect()
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
    fn test_get_variables_list() {
        unsafe { env::set_var("TEST_GET_VARIABLES", "test_value") };
        let list = get_variables();
        assert!(list.contains(&("TEST_GET_VARIABLES".to_string(), "test_value".to_string())));
        unsafe { env::remove_var("TEST_GET_VARIABLES") };
    }

    #[test]
    fn test_set_variable_simple() {
        let result = set_variable("TEST_VAR", "test_value", false, None);
        assert!(result.is_ok());
        assert_eq!(env::var("TEST_VAR").unwrap(), "test_value");
        unsafe { env::remove_var("TEST_VAR") };
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
        unsafe { env::remove_var("TEST_PROC_VAR") };
    }

    #[test]
    fn test_print_env() {
        unsafe { env::set_var("TEST_PRINT_VAR", "test_value") };
        let mut buffer = vec![];
        print_env("{name} = \"{value}\"", &mut buffer);
        assert!(
            String::from_utf8(buffer)
                .unwrap()
                .contains("TEST_PRINT_VAR = \"test_value\"")
        );
        unsafe { env::remove_var("TEST_PRINT_VAR") };
    }

    #[test]
    fn test_set_variable_invalid_process() {
        let result = set_variable(
            "TEST_INVALID_PROC",
            "test_value",
            false,
            Some("nonexistent_command".to_string()),
        );

        // Check that the operation failed
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ErrorKind::ProcessFailed));

        // Check that the variable is still set despite process failure
        assert_eq!(env::var("TEST_INVALID_PROC").unwrap(), "test_value");

        // Cleanup
        unsafe { env::remove_var("TEST_INVALID_PROC") };
    }

    #[test]
    fn test_delete_variable() {
        unsafe { env::set_var("TEST_DELETE_VAR", "test_value") };
        let result = delete_variable("TEST_DELETE_VAR".to_string(), false);
        assert!(result.is_ok());
        assert!(env::var("TEST_DELETE_VAR").is_err());
    }

    #[test]
    fn test_set_variable_empty_value() {
        let result = set_variable("TEST_EMPTY_VAR", "", false, None);
        assert!(result.is_ok());
        assert_eq!(env::var("TEST_EMPTY_VAR").unwrap(), "");
        unsafe { env::remove_var("TEST_EMPTY_VAR") };
    }

    #[test]
    fn test_print_env_format() {
        // Set up test environment variables
        unsafe { env::set_var("TEST_VAR_1", "value1") };
        unsafe { env::set_var("TEST_VAR_2", "value2") };

        let mut buffer = vec![];
        print_env("{name} = \"{value}\"", &mut buffer);
        assert!(
            String::from_utf8(buffer.clone())
                .unwrap()
                .contains("TEST_VAR_1 = \"value1\"")
        );
        assert!(
            String::from_utf8(buffer)
                .unwrap()
                .contains("TEST_VAR_2 = \"value2\"")
        );

        // Clean up
        unsafe { env::remove_var("TEST_VAR_1") };
        unsafe { env::remove_var("TEST_VAR_2") };
    }

    #[test]
    fn test_print_env_empty_value() {
        unsafe { env::set_var("TEST_EMPTY", "") };

        let mut buffer = vec![];
        print_env("{name} = \"{value}\"", &mut buffer);
        assert!(
            String::from_utf8(buffer)
                .unwrap()
                .contains("TEST_EMPTY = \"\"")
        );

        unsafe { env::remove_var("TEST_EMPTY") };
    }

    #[test]
    fn test_print_env_special_characters() {
        unsafe { env::set_var("TEST_SPECIAL", "value with spaces and $#@!") };

        let mut buffer = vec![];
        print_env("{name} = \"{value}\"", &mut buffer);
        assert!(
            String::from_utf8(buffer)
                .unwrap()
                .contains("TEST_SPECIAL = \"value with spaces and $#@!\"")
        );

        unsafe { env::remove_var("TEST_SPECIAL") };
    }

    #[test]
    fn test_set_variable_global() {
        let result = set_variable("TEST_GLOBAL_VAR", "test_value", true, None);
        match result {
            Ok(_) => {
                assert_eq!(env::var("TEST_GLOBAL_VAR").unwrap(), "test_value");
                delete_variable("TEST_GLOBAL_VAR".to_string(), true).unwrap();
            }
            Err(ErrorKind::CannotSetVariableGlobally(_)) => {
                // Test passes if we get permission error on non-admin run
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn test_set_variable_global_with_process() {
        #[cfg(windows)]
        let cmd = "cmd /C echo test";
        #[cfg(not(windows))]
        let cmd = "echo test";

        let result = set_variable(
            "TEST_GLOBAL_PROC",
            "test_value",
            true,
            Some(cmd.to_string()),
        );
        match result {
            Ok(_) => {
                assert_eq!(env::var("TEST_GLOBAL_PROC").unwrap(), "test_value");
                delete_variable("TEST_GLOBAL_PROC".to_string(), true).unwrap();
            }
            Err(ErrorKind::CannotSetVariableGlobally(_)) => {
                // Test passes if we get permission error on non-admin run
            }
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
            Ok(_) => {}
            Err(ErrorKind::CannotDeleteVariableGlobally(_)) => {
                // Test passes if we get permission error on non-admin run
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }
}
