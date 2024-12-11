use colored::Colorize;
use std::process;
use subprocess::Exec;

/// Runs given command using system shell
pub fn run(process: String, exit_on_error: bool) {
    let result = Exec::shell(process).join().unwrap_or_else(|_| {
        error("can't start process", exit_on_error);
        // Exit with non-zero exit code if we can't start process
        process::exit(1);
    });

    // Exit with non-zero exit code if process did not successful
    if !result.success() {
        process::exit(1);
    }
}

/// Print info about error
pub fn error(text: &str, exit_on_error: bool) {
    eprintln!("{} {}", "error:".red(), text);
    if exit_on_error {
        process::exit(1);
    }
}

/// Print info about warning
pub fn warning(text: &str) {
    eprintln!("{} {}", "warning:".yellow(), text);
}
