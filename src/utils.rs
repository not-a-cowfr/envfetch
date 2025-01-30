use colored::Colorize;
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
    process::exit(1);
}

/// Print info about warning
pub fn warning(text: &str, exit_on_warning: bool) {
    eprintln!("{} {}", "warning:".yellow(), text);
    if exit_on_warning {
        process::exit(1);
    }
}
