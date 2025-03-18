use log::error;
use log::warn;
use rayon::prelude::*;
use std::process::ExitCode;
use std::{env, fs};

use crate::interactive::InteractiveMode;
use crate::models::*;
use crate::utils::*;
use crate::variables;

/// Run tool's command
pub fn run_command(command: &Commands) -> ExitCode {
    match command {
        Commands::Get(opt) => {
            if let Err(error) = get(opt) {
                error!("{}", error);
                if let ErrorKind::CannotFindVariable(key, no_similar_names) = error {
                    if !no_similar_names {
                        let similar_names = find_similar_string(
                            key.clone(),
                            env::vars().map(|(key, _)| key).collect(),
                            0.6,
                        );
                        if !similar_names.is_empty() {
                            eprintln!("Did you mean:");
                            for name in similar_names {
                                eprintln!("  {}", &name);
                            }
                        }
                    }
                }
                return ExitCode::FAILURE;
            }
        }
        Commands::Print(opt) => print_env(opt),
        Commands::Load(opt) => {
            if let Err(error) = load(opt) {
                error!("{}", error);
                return ExitCode::FAILURE;
            }
        }
        Commands::Set(opt) => {
            if let Err(error) = set(opt) {
                error!("{}", error);
                return ExitCode::FAILURE;
            }
        }
        Commands::Add(opt) => {
            if let Err(error) = add(opt) {
                error!("{}", error);
                return ExitCode::FAILURE;
            }
        }
        Commands::Delete(opt) => {
            if let Err(error) = delete(opt) {
                error!("{}", error);
                return ExitCode::FAILURE;
            }
        }
        Commands::Interactive => {
            let mut terminal = ratatui::init();
            let result = InteractiveMode::init().run(&mut terminal);
            ratatui::restore();
            if let Err(error) = result {
                error!("{}", error);
                return ExitCode::FAILURE;
            }
        }
    }
    ExitCode::SUCCESS
}

/// Print all environment variables
pub fn print_env(opt: &PrintArgs) {
    let format = &opt.format.clone().unwrap_or("{name} = \"{value}\"".to_owned());
    // Print all environment variables
    variables::print_env(format);
}

/// Load variables from dotenv-style file
pub fn load(args: &LoadArgs) -> Result<(), ErrorKind> {
    // Try to read file
    match fs::read_to_string(&args.file) {
        Ok(content) => {
            // Try to parse file
            match dotenv_parser::parse_dotenv(&content) {
                Ok(variables) => {
                    variables.into_par_iter().try_for_each(
                        |(key, value)| -> Result<(), ErrorKind> {
                            variables::set_variable(&key, &value, args.global, args.process.clone())
                        },
                    )?;
                    if let Some(process) = args.process.clone() {
                        return run(process, false);
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
            return Err(ErrorKind::CannotFindVariable(
                args.key.clone(),
                args.no_similar_names,
            ));
        }
    }
    Ok(())
}

/// Set value to environment variable
pub fn set(args: &SetArgs) -> Result<(), ErrorKind> {
    validate_var_name(&args.key).map_err(ErrorKind::NameValidationError)?;

    variables::set_variable(&args.key, &args.value, args.global, args.process.clone())?;
    Ok(())
}

/// Add value to environment variable
pub fn add(args: &AddArgs) -> Result<(), ErrorKind> {
    validate_var_name(&args.key).map_err(ErrorKind::NameValidationError)?;

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
    validate_var_name(&args.key).map_err(ErrorKind::NameValidationError)?;

    // Check if variable exists
    match env::var(&args.key) {
        Ok(_) => {
            variables::delete_variable(args.key.clone(), args.global)?;
        }
        _ => {
            warn!("{}", "variable doesn't exists");
        }
    }
    if let Some(process) = args.process.clone() {
        return run(process, false);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use tempfile::NamedTempFile;

    use super::*;
    use crate::utils::with_captured_output;
    use std::io::Write;

    #[test]
    fn test_run_command_get_success() {
        unsafe { env::set_var("TEST_RUN_VAR", "test_value") };
        with_captured_output(|| {
            run_command(&Commands::Get(GetArgs {
                key: "TEST_RUN_VAR".to_string(),
                no_similar_names: false,
            }));
        });
        unsafe { env::remove_var("TEST_RUN_VAR") };
    }

    #[test]
    fn test_run_command_get_fail() {
        with_captured_output(|| {
            assert_eq!(
                run_command(&Commands::Get(GetArgs {
                    key: "TEST_RUN_VAR_awzsenfkaqyG".to_string(),
                    no_similar_names: false,
                })),
                ExitCode::FAILURE
            );
        });
    }

    #[test]
    fn test_run_command_set() {
        with_captured_output(|| {
            run_command(&Commands::Set(SetArgs {
                key: "TEST_SET_RUN".to_string(),
                value: "test_value".to_string(),
                global: false,
                process: None,
            }));
        });

        assert_eq!(env::var("TEST_SET_RUN").unwrap(), "test_value");
        unsafe { env::remove_var("TEST_SET_RUN") };
    }

    #[test]
    fn test_run_command_add() {
        unsafe { env::set_var("TEST_ADD_RUN", "initial_") };

        with_captured_output(|| {
            run_command(&Commands::Add(AddArgs {
                key: "TEST_ADD_RUN".to_string(),
                value: "value".to_string(),
                global: false,
                process: None,
            }));
        });

        assert_eq!(env::var("TEST_ADD_RUN").unwrap(), "initial_value");
        unsafe { env::remove_var("TEST_ADD_RUN") };
    }

    #[test]
    fn test_run_command_print() {
        unsafe { env::set_var("TEST_PRINT_RUN", "test_value") };
        with_captured_output(|| {
            run_command(&Commands::Print(PrintArgs { format: None }));
        });
        unsafe { env::remove_var("TEST_PRINT_RUN") };
    }

    #[test]
    fn test_run_command_print_with_format() {
        unsafe { env::set_var("TEST_PRINT_RUN", "test_value") };
        with_captured_output(|| {
            run_command(&Commands::Print(PrintArgs { format: Some("{name} = {value}".to_owned()) }));
        });
        unsafe { env::remove_var("TEST_PRINT_RUN") };
    }

    #[test]
    fn test_run_command_delete() {
        unsafe { env::set_var("TEST_DELETE_RUN", "test_value") };

        with_captured_output(|| {
            run_command(&Commands::Delete(DeleteArgs {
                key: "TEST_DELETE_RUN".to_string(),
                global: false,
                process: None,
            }));
        });

        assert!(env::var("TEST_DELETE_RUN").is_err());
    }

    #[test]
    fn test_run_command_load() {
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        writeln!(temp_file, "TEST_LOAD_RUN=test_value").unwrap();

        with_captured_output(|| {
            run_command(&Commands::Load(LoadArgs {
                file: temp_file.path().to_string_lossy().to_string(),
                global: false,
                process: None,
            }));
        });

        assert_eq!(env::var("TEST_LOAD_RUN").unwrap(), "test_value");
        unsafe { env::remove_var("TEST_LOAD_RUN") };
    }

    #[test]
    fn test_print_env_command() {
        // Set test variable
        unsafe { env::set_var("TEST_PRINT_VAR", "test_value") };

        // Call function - just verify it executes without panicking
        print_env(&PrintArgs { format: None });

        // Clean up
        unsafe { env::remove_var("TEST_PRINT_VAR") };
    }

    #[test]
    fn test_print_env_multiple_variables() {
        // Set test variables
        unsafe { env::set_var("TEST_VAR_1", "value1") };
        unsafe { env::set_var("TEST_VAR_2", "value2") };

        // Call function - just verify it executes without panicking
        print_env(&PrintArgs { format: None });

        // Clean up
        unsafe { env::remove_var("TEST_VAR_1") };
        unsafe { env::remove_var("TEST_VAR_2") };
    }

    #[test]
    fn test_get_existing_variable() {
        unsafe { env::set_var("TEST_GET_VAR", "test_value") };

        let args = GetArgs {
            key: "TEST_GET_VAR".to_string(),
            no_similar_names: false,
        };

        let result = get(&args);
        assert!(result.is_ok());

        unsafe { env::remove_var("TEST_GET_VAR") };
    }

    #[test]
    fn test_get_nonexistent_variable_with_similar_names() {
        unsafe { env::set_var("TEST_SIMILAR", "value") };

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
            }
            _ => panic!("Unexpected error type"),
        }

        unsafe { env::remove_var("TEST_SIMILAR") };
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
            }
            _ => panic!("Unexpected error type"),
        }
    }

    #[test]
    fn test_get_special_characters() {
        unsafe { env::set_var("TEST_SPECIAL_$#@", "special_value") };

        let args = GetArgs {
            key: "TEST_SPECIAL_$#@".to_string(),
            no_similar_names: false,
        };

        let result = get(&args);
        assert!(result.is_ok());

        unsafe { env::remove_var("TEST_SPECIAL_$#@") };
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
        unsafe { env::remove_var("TEST_SET_VAR") };
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
            }
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
            }
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

        unsafe { env::remove_var("TEST_PROCESS_VAR") };
    }

    #[test]
    fn test_set_overwrite_existing() {
        unsafe { env::set_var("TEST_OVERWRITE", "old_value") };

        let args = SetArgs {
            key: "TEST_OVERWRITE".to_string(),
            value: "new_value".to_string(),
            global: false,
            process: None,
        };

        let result = set(&args);
        assert!(result.is_ok());

        assert_eq!(env::var("TEST_OVERWRITE").unwrap(), "new_value");
        unsafe { env::remove_var("TEST_OVERWRITE") };
    }

    #[test]
    fn test_add_to_nonexistent_variable() {
        let args = AddArgs {
            key: "TEST_ADD_NEW".to_string(),
            value: "new_value".to_string(),
            global: false,
            process: None,
        };

        let result = add(&args);
        assert!(result.is_ok());
        assert_eq!(env::var("TEST_ADD_NEW").unwrap(), "new_value");
        unsafe { env::remove_var("TEST_ADD_NEW") };
    }

    #[test]
    fn test_add_to_existing_variable() {
        unsafe { env::set_var("TEST_ADD_EXISTING", "existing_") };

        let args = AddArgs {
            key: "TEST_ADD_EXISTING".to_string(),
            value: "appended".to_string(),
            global: false,
            process: None,
        };

        let result = add(&args);
        assert!(result.is_ok());
        assert_eq!(env::var("TEST_ADD_EXISTING").unwrap(), "existing_appended");
        unsafe { env::remove_var("TEST_ADD_EXISTING") };
    }

    #[test]
    fn test_add_with_invalid_name() {
        let args = AddArgs {
            key: "INVALID NAME".to_string(),
            value: "test_value".to_string(),
            global: false,
            process: None,
        };

        let result = add(&args);
        assert!(result.is_err());
        match result.unwrap_err() {
            ErrorKind::NameValidationError(err) => {
                assert!(err.contains("cannot contain spaces"));
            }
            _ => panic!("Unexpected error type"),
        }
    }

    #[test]
    fn test_add_empty_value() {
        unsafe { env::set_var("TEST_ADD_EMPTY", "existing") };

        let args = AddArgs {
            key: "TEST_ADD_EMPTY".to_string(),
            value: "".to_string(),
            global: false,
            process: None,
        };

        let result = add(&args);
        assert!(result.is_ok());
        assert_eq!(env::var("TEST_ADD_EMPTY").unwrap(), "existing");
        unsafe { env::remove_var("TEST_ADD_EMPTY") };
    }

    #[test]
    fn test_add_with_process() {
        let args = AddArgs {
            key: "TEST_ADD_PROCESS".to_string(),
            value: "_value".to_string(),
            global: false,
            process: Some("echo test".to_string()),
        };

        unsafe { env::set_var("TEST_ADD_PROCESS", "initial") };
        let result = add(&args);
        assert!(result.is_ok());
        assert_eq!(env::var("TEST_ADD_PROCESS").unwrap(), "initial_value");
        unsafe { env::remove_var("TEST_ADD_PROCESS") };
    }

    #[test]
    fn test_delete_existing_variable() {
        unsafe { env::set_var("TEST_DELETE_VAR", "test_value") };

        let args = DeleteArgs {
            key: "TEST_DELETE_VAR".to_string(),
            global: false,
            process: None,
        };

        let result = delete(&args);
        assert!(result.is_ok());
        assert!(env::var("TEST_DELETE_VAR").is_err());
    }

    #[test]
    fn test_delete_nonexistent_variable() {
        let args = DeleteArgs {
            key: "NONEXISTENT_VAR".to_string(),
            global: false,
            process: None,
        };

        let result = delete(&args);
        // Should succeed even if variable doesn't exist
        assert!(result.is_ok());
    }

    #[test]
    fn test_delete_with_invalid_name() {
        let args = DeleteArgs {
            key: "INVALID NAME".to_string(),
            global: false,
            process: None,
        };

        let result = delete(&args);
        assert!(result.is_err());
        match result.unwrap_err() {
            ErrorKind::NameValidationError(err) => {
                assert!(err.contains("cannot contain spaces"));
            }
            _ => panic!("Unexpected error type"),
        }
    }

    #[test]
    fn test_delete_with_process() {
        unsafe { env::set_var("TEST_DELETE_PROCESS", "test_value") };

        let args = DeleteArgs {
            key: "TEST_DELETE_PROCESS".to_string(),
            global: false,
            process: Some("echo test".to_string()),
        };

        let result = delete(&args);
        assert!(result.is_ok());
        assert!(env::var("TEST_DELETE_PROCESS").is_err());
    }

    #[test]
    fn test_delete_with_empty_name() {
        let args = DeleteArgs {
            key: "".to_string(),
            global: false,
            process: None,
        };

        let result = delete(&args);
        assert!(result.is_err());
        match result.unwrap_err() {
            ErrorKind::NameValidationError(err) => {
                assert!(err.contains("empty"));
            }
            _ => panic!("Unexpected error type"),
        }
    }

    #[test]
    fn test_load_valid_env_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "TEST_VAR=test_value\nOTHER_VAR=other_value").unwrap();

        let args = LoadArgs {
            file: temp_file.path().to_string_lossy().to_string(),
            global: false,
            process: None,
        };

        let result = load(&args);
        assert!(result.is_ok());
        assert_eq!(env::var("TEST_VAR").unwrap(), "test_value");
        assert_eq!(env::var("OTHER_VAR").unwrap(), "other_value");

        unsafe { env::remove_var("TEST_VAR") };
        unsafe { env::remove_var("OTHER_VAR") };
    }

    #[test]
    fn test_load_nonexistent_file() {
        let args = LoadArgs {
            file: "nonexistent.env".to_string(),
            global: false,
            process: None,
        };

        let result = load(&args);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ErrorKind::FileError(_)));
    }

    #[test]
    fn test_load_invalid_env_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        // Using invalid .env format that dotenv_parser will reject
        writeln!(temp_file, "TEST_VAR test_value").unwrap();

        let args = LoadArgs {
            file: temp_file.path().to_string_lossy().to_string(),
            global: false,
            process: None,
        };

        let result = load(&args);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ErrorKind::ParsingError(_)));
    }

    #[test]
    fn test_load_with_process() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "TEST_PROCESS_VAR=process_value").unwrap();

        #[cfg(windows)]
        let cmd = "cmd /C echo test"; // Simple echo command for Windows
        #[cfg(not(windows))]
        let cmd = "echo test"; // Simple echo command for Unix

        let args = LoadArgs {
            file: temp_file.path().to_string_lossy().to_string(),
            global: false,
            process: Some(cmd.to_string()),
        };

        // First verify the variable is set correctly
        let result = load(&args);
        assert!(result.is_ok(), "Load operation failed: {:?}", result);
    }

    #[test]
    fn test_load_empty_file() {
        let temp_file = NamedTempFile::new().unwrap();

        let args = LoadArgs {
            file: temp_file.path().to_string_lossy().to_string(),
            global: false,
            process: None,
        };

        let result = load(&args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_load_with_invalid_variable_name() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "TEST_VAR=test_value\nINVALID NAME=value").unwrap();

        let args = LoadArgs {
            file: temp_file.path().to_string_lossy().to_string(),
            global: false,
            process: None,
        };

        let result = load(&args);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ErrorKind::ParsingError(_)));
    }

    #[test]
    fn test_run_command_print_env() {
        unsafe { env::set_var("TEST_PRINT_ENV", "test_value") };
        with_captured_output(|| {
            assert_eq!(run_command(&Commands::Print(PrintArgs { format: None })), ExitCode::SUCCESS);
        });
        unsafe { env::remove_var("TEST_PRINT_ENV") };
    }

    #[test]
    fn test_run_command_get_with_similar_names() {
        unsafe { env::set_var("TEST_SIMILAR_VAR", "value") };
        with_captured_output(|| {
            assert_eq!(
                run_command(&Commands::Get(GetArgs {
                    key: "TEST_SMILAR_VAR".to_string(), // Intentional typo
                    no_similar_names: false,
                })),
                ExitCode::FAILURE
            );
        });
        unsafe { env::remove_var("TEST_SIMILAR_VAR") };
    }

    #[test]
    fn test_run_command_set_with_process() {
        with_captured_output(|| {
            assert_eq!(
                run_command(&Commands::Set(SetArgs {
                    key: "TEST_SET_PROC".to_string(),
                    value: "test_value".to_string(),
                    global: false,
                    process: Some("echo test".to_string()),
                })),
                ExitCode::SUCCESS
            );
        });
        assert_eq!(env::var("TEST_SET_PROC").unwrap(), "test_value");
        unsafe { env::remove_var("TEST_SET_PROC") };
    }

    #[test]
    fn test_run_command_set_invalid_name() {
        with_captured_output(|| {
            assert_eq!(
                run_command(&Commands::Set(SetArgs {
                    key: "INVALID NAME".to_string(),
                    value: "test_value".to_string(),
                    global: false,
                    process: None,
                })),
                ExitCode::FAILURE
            );
        });
    }

    #[test]
    fn test_run_command_add_to_existing() {
        unsafe { env::set_var("TEST_ADD_EXISTING", "initial_") };
        with_captured_output(|| {
            assert_eq!(
                run_command(&Commands::Add(AddArgs {
                    key: "TEST_ADD_EXISTING".to_string(),
                    value: "appended".to_string(),
                    global: false,
                    process: None,
                })),
                ExitCode::SUCCESS
            );
        });
        assert_eq!(env::var("TEST_ADD_EXISTING").unwrap(), "initial_appended");
        unsafe { env::remove_var("TEST_ADD_EXISTING") };
    }

    #[test]
    fn test_run_command_add_with_invalid_name() {
        with_captured_output(|| {
            assert_eq!(
                run_command(&Commands::Add(AddArgs {
                    key: "INVALID NAME".to_string(),
                    value: "test_value".to_string(),
                    global: false,
                    process: None,
                })),
                ExitCode::FAILURE
            );
        });
    }

    #[test]
    fn test_run_command_delete_nonexistent() {
        with_captured_output(|| {
            assert_eq!(
                run_command(&Commands::Delete(DeleteArgs {
                    key: "NONEXISTENT_VAR".to_string(),
                    global: false,
                    process: None,
                })),
                ExitCode::SUCCESS // Should succeed even if var doesn't exist
            );
        });
    }

    #[test]
    fn test_run_command_load_nonexistent_file() {
        with_captured_output(|| {
            assert_eq!(
                run_command(&Commands::Load(LoadArgs {
                    file: "nonexistent.env".to_string(),
                    global: false,
                    process: None,
                })),
                ExitCode::FAILURE
            );
        });
    }

    #[test]
    fn test_run_command_load_with_process() {
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        writeln!(temp_file, "TEST_LOAD_PROC=test_value").unwrap();

        with_captured_output(|| {
            assert_eq!(
                run_command(&Commands::Load(LoadArgs {
                    file: temp_file.path().to_string_lossy().to_string(),
                    global: false,
                    process: Some("echo test".to_string()),
                })),
                ExitCode::SUCCESS
            );
        });
        assert_eq!(env::var("TEST_LOAD_PROC").unwrap(), "test_value");
        unsafe { env::remove_var("TEST_LOAD_PROC") };
    }

    #[test]
    fn test_run_command_global_operations() {
        with_captured_output(|| {
            let result = run_command(&Commands::Set(SetArgs {
                key: "TEST_GLOBAL".to_string(),
                value: "test_value".to_string(),
                global: true,
                process: None,
            }));
            // Test passes if operation succeeds OR fails with permission error
            match result {
                ExitCode::SUCCESS => {
                    assert_eq!(env::var("TEST_GLOBAL").unwrap(), "test_value");
                    assert_eq!(
                        run_command(&Commands::Delete(DeleteArgs {
                            key: "TEST_GLOBAL".to_string(),
                            global: true,
                            process: None,
                        })),
                        ExitCode::SUCCESS
                    );
                }
                ExitCode::FAILURE => {} // Expected on non-admin
                _ => panic!("Unexpected exit code"),
            }
        });
    }

    #[test]
    fn test_run_command_delete_with_process_fail() {
        unsafe { env::set_var("TEST_DELETE_PROC_FAIL", "test_value") };
        with_captured_output(|| {
            assert_eq!(
                run_command(&Commands::Delete(DeleteArgs {
                    key: "TEST_DELETE_PROC_FAIL".to_string(),
                    global: false,
                    process: Some("nonexistent_command_123".to_string()),
                })),
                ExitCode::FAILURE
            );
        });
        // Variable should still be deleted even if process fails
        assert!(env::var("TEST_DELETE_PROC_FAIL").is_err());
    }

    #[test]
    fn test_run_command_delete_invalid_name() {
        with_captured_output(|| {
            assert_eq!(
                run_command(&Commands::Delete(DeleteArgs {
                    key: "INVALID NAME".to_string(),
                    global: false,
                    process: None,
                })),
                ExitCode::FAILURE
            );
        });
    }

    #[test]
    fn test_run_command_delete_empty_name() {
        with_captured_output(|| {
            assert_eq!(
                run_command(&Commands::Delete(DeleteArgs {
                    key: "".to_string(),
                    global: false,
                    process: None,
                })),
                ExitCode::FAILURE
            );
        });
    }
}
