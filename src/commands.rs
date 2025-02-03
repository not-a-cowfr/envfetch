use rayon::prelude::*;
use std::io::stdout;
use std::{env, fs};
use log::warn;

use crate::models::*;
use crate::utils::*;
use crate::variables;

/// Print all environment variables
pub fn print_env() {
    // Print all environment variables
    variables::print_env(&mut stdout());
}

/// Load variables from dotenv-style file
pub fn load(args: &LoadArgs) -> Result<(), ErrorKind> {
    // Try to read file
    match fs::read_to_string(&args.file) {
        Ok(content) => {
            // Try to parse file
            match dotenv_parser::parse_dotenv(&content) {
                Ok(variables) => {
                    variables.into_par_iter().try_for_each(|(key, value)| -> Result<(), ErrorKind> {
                        return variables::set_variable(&key, &value, args.global, args.process.clone())
                    })?;
                    if let Some(process) = args.process.clone() {
                        return run(process);
                    }
                }
                Err(err) => {
                    return Err(ErrorKind::ParsingError(err.to_string()));
                }
            }
        }
        Err(err) => {
            return Err(ErrorKind::FileError(err.to_string()));
        }
    }
    Ok(())
}

/// Get value of variable
pub fn get(args: &GetArgs) -> Result<(), ErrorKind> {
    // Check if variable with specified name exists
    match env::var(&args.key) {
        Ok(value) => println!("{:?}", &value),
        // If variable not found
        _ => {
            return Err(ErrorKind::CannotFindVariable(args.key.clone(), args.no_similar_names));
        }
    }
    Ok(())
}

/// Set value to environment variable
pub fn set(args: &SetArgs) -> Result<(), ErrorKind> {
    validate_var_name(&args.key).map_err(|err| ErrorKind::NameValidationError(err))?;

    variables::set_variable(&args.key, &args.value, args.global, args.process.clone())?;
    Ok(())
}

/// Add value to environment variable
pub fn add(args: &AddArgs) -> Result<(), ErrorKind> {
    validate_var_name(&args.key).map_err(|err| ErrorKind::NameValidationError(err))?;

    let current_value = if let Ok(value) = env::var(&args.key) {
        value
    } else {
        "".to_string()
    };

    variables::set_variable(
        &args.key,
        &format!("{}{}", current_value, args.value),
        args.global,
        args.process.clone(),
    )?;
    Ok(())
}

/// Delete environment variable
pub fn delete(args: &DeleteArgs) -> Result<(), ErrorKind> {
    validate_var_name(&args.key).map_err(|err| ErrorKind::NameValidationError(err))?;

    // Check if variable exists
    match env::var(&args.key) {
        Ok(_) => {
            variables::delete_variable(args.key.clone(), args.global)?;
        }
        _ => {
            warn!("{}", "variable doesn't exists");
        },
    }
    if let Some(process) = args.process.clone() {
        return run(process);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    // Helper function to capture stdout
    // TODO: use more efficient method
    fn capture_stdout<F>(_f: F) -> String 
    where
        F: FnOnce(),
    {
        let mut stdout = Vec::new();
        {
            let mut cursor = Cursor::new(&mut stdout);
            let _old_stdout = std::io::stdout();
            // Temporarily redirect stdout to our buffer
            variables::print_env(&mut cursor);
        }
        String::from_utf8(stdout).unwrap()
    }

    #[test]
    fn test_print_env_command() {
        // Set up test environment
        env::set_var("TEST_PRINT_CMD", "test_value");
        
        // Capture and verify output
        let output = capture_stdout(|| {
            print_env();
        });
        
        // Verify output contains our test variable
        assert!(output.contains("TEST_PRINT_CMD"));
        assert!(output.contains("test_value"));
        
        // Clean up
        env::remove_var("TEST_PRINT_CMD");
    }

    #[test]
    fn test_print_env_multiple_variables() {
        // Set up test environment
        env::set_var("TEST_VAR_1", "value1");
        env::set_var("TEST_VAR_2", "value2");
        
        let output = capture_stdout(|| {
            print_env();
        });
        
        // Verify both variables are present
        assert!(output.contains("TEST_VAR_1"));
        assert!(output.contains("value1"));
        assert!(output.contains("TEST_VAR_2"));
        assert!(output.contains("value2"));
        
        // Clean up
        env::remove_var("TEST_VAR_1");
        env::remove_var("TEST_VAR_2");
    }

    #[test]
    fn test_get_existing_variable() {
        env::set_var("TEST_GET_VAR", "test_value");
        
        let args = GetArgs {
            key: "TEST_GET_VAR".to_string(),
            no_similar_names: false,
        };
        
        let result = get(&args);
        assert!(result.is_ok());
        
        env::remove_var("TEST_GET_VAR");
    }

    #[test]
    fn test_get_nonexistent_variable_with_similar_names() {
        env::set_var("TEST_SIMILAR", "value");
        
        let args = GetArgs {
            key: "TEST_SMILAR".to_string(), // Intentional typo
            no_similar_names: false,
        };
        
        let result = get(&args);
        assert!(result.is_err());
        match result.unwrap_err() {
            ErrorKind::CannotFindVariable(var, no_similar) => {
                assert_eq!(var, "TEST_SMILAR");
                assert!(!no_similar);
            },
            _ => panic!("Unexpected error type"),
        }
        
        env::remove_var("TEST_SIMILAR");
    }

    #[test]
    fn test_get_nonexistent_variable_no_similar_names() {
        let args = GetArgs {
            key: "NONEXISTENT_VAR".to_string(),
            no_similar_names: true,
        };
        
        let result = get(&args);
        assert!(result.is_err());
        match result.unwrap_err() {
            ErrorKind::CannotFindVariable(var, no_similar) => {
                assert_eq!(var, "NONEXISTENT_VAR");
                assert!(no_similar);
            },
            _ => panic!("Unexpected error type"),
        }
    }

    #[test]
    fn test_get_special_characters() {
        env::set_var("TEST_SPECIAL_$#@", "special_value");
        
        let args = GetArgs {
            key: "TEST_SPECIAL_$#@".to_string(),
            no_similar_names: false,
        };
        
        let result = get(&args);
        assert!(result.is_ok());
        
        env::remove_var("TEST_SPECIAL_$#@");
    }

    #[test]
    fn test_set_valid_variable() {
        let args = SetArgs {
            key: "TEST_SET_VAR".to_string(),
            value: "test_value".to_string(),
            global: false,
            process: None,
        };
        
        let result = set(&args);
        assert!(result.is_ok());
        
        assert_eq!(env::var("TEST_SET_VAR").unwrap(), "test_value");
        env::remove_var("TEST_SET_VAR");
    }

    #[test]
    fn test_set_invalid_variable_name() {
        let args = SetArgs {
            key: "INVALID NAME".to_string(), // Space in name
            value: "test_value".to_string(),
            global: false,
            process: None,
        };
        
        let result = set(&args);
        assert!(result.is_err());
        match result.unwrap_err() {
            ErrorKind::NameValidationError(err) => {
                assert!(err.contains("cannot contain spaces"));
            },
            _ => panic!("Unexpected error type"),
        }
    }

    #[test]
    fn test_set_empty_variable_name() {
        let args = SetArgs {
            key: "".to_string(),
            value: "test_value".to_string(),
            global: false,
            process: None,
        };
        
        let result = set(&args);
        assert!(result.is_err());
        match result.unwrap_err() {
            ErrorKind::NameValidationError(err) => {
                assert!(err.contains("cannot be empty"));
            },
            _ => panic!("Expected NameValidationError"),
        }
        
        // Verify variable was not set
        assert!(env::var("").is_err());
    }

    #[test]
    fn test_set_with_process() {
        let args = SetArgs {
            key: "TEST_PROCESS_VAR".to_string(),
            value: "test_value".to_string(),
            global: false,
            process: Some("echo test".to_string()),
        };
        
        let result = set(&args);
        assert!(result.is_ok());
        
        assert_eq!(env::var("TEST_PROCESS_VAR").unwrap(), "test_value");
        env::remove_var("TEST_PROCESS_VAR");
    }

    #[test]
    fn test_set_overwrite_existing() {
        env::set_var("TEST_OVERWRITE", "old_value");
        
        let args = SetArgs {
            key: "TEST_OVERWRITE".to_string(),
            value: "new_value".to_string(),
            global: false,
            process: None,
        };
        
        let result = set(&args);
        assert!(result.is_ok());
        
        assert_eq!(env::var("TEST_OVERWRITE").unwrap(), "new_value");
        env::remove_var("TEST_OVERWRITE");
    }
}
