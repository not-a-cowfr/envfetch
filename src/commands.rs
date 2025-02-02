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
                        return variables::set_variable(key.as_str(), value.as_str(), args.global, args.process.clone())
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
