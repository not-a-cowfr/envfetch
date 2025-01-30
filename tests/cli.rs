//! Integration tests for CLI

use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use predicates::prelude::*;
use std::env;
use std::process::Command;

#[test]
/// Test for set command if specified process is successful
/// Check if variable is set and envfetch exits with 0
/// We check it separately for Windows and Unix, because commands are different
fn set_command_success() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("envfetch")?;
    cmd.arg("set").arg("MY_VAR").arg("Hello");
    // Windows
    #[cfg(target_os = "windows")]
    cmd.arg("echo %MY_VAR%")
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello"));

    // Linux and macOS
    #[cfg(not(target_os = "windows"))]
    cmd.arg("echo $MY_VAR")
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello"));
    Ok(())
}

#[test]
/// Test for set command if specified process is not successful
/// Check if envfetch exits with non-zero exit code
fn set_command_failure() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("envfetch")?;
    cmd.arg("set").arg("MY_VARR").arg("Hello");
    // We can use only Windows command here because it should fail
    cmd.arg("%MY_VARIABLE%").assert().failure();
    Ok(())
}

#[test]
/// Test for get command if specified variable exists
fn get_variable_exists() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("envfetch")?;
    env::set_var("MY_VAR", "Hello");
    cmd.arg("get").arg("MY_VAR");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Hello"));
    Ok(())
}

#[test]
/// Test for get command if specified variable doesn't exist
fn get_variable_doesnt_exists() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("envfetch")?;
    cmd.arg("get").arg("MY_VARIABLE");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("error: can't find 'MY_VARIABLE'"));
    Ok(())
}

#[test]
/// Test for get command if specified variable doesn't exist and showing similar variables is enabled
fn get_variable_doesnt_exists_similar_enabled() -> Result<(), Box<dyn std::error::Error>> {
    env::set_var("MY_VARIABLEE", "Hello");
    let mut cmd = Command::cargo_bin("envfetch")?;
    cmd.arg("get").arg("MY_VARIABLE");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("error: can't find 'MY_VARIABLE'"))
        .stderr(predicate::str::contains("Did you mean:"))
        .stderr(predicate::str::contains("MY_VARIABLEE"));
    Ok(())
}

#[test]
/// Test for get command if specified variable doesn't exist and showing similar variables is disabled
fn get_variable_doesnt_exists_similar_disabled() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("envfetch")?;
    env::set_var("MY_VARIABLEE", "Hello");
    cmd.arg("get").arg("MY_VARIABLE").arg("--no-similar-names");
    cmd.assert().failure();
    Ok(())
}

#[test]
/// Test for print command
fn print_success() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("envfetch")?;
    env::set_var("PRINT_TEST", "Print");
    cmd.arg("print")
        .assert()
        .success()
        .stdout(predicate::str::contains("PRINT_TEST = \"Print\""));
    Ok(())
}

#[test]
/// Test for delete command if specified process is successful
fn delete_command_success() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("envfetch")?;
    env::set_var("MY_VAR", "Hello");
    cmd.arg("delete").arg("MY_VAR");
    // Windows
    #[cfg(target_os = "windows")]
    cmd.arg("echo 'Hello'")
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello"));

    // Linux and macOS
    #[cfg(not(target_os = "windows"))]
    cmd.arg("echo 'Hello'")
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello"));
    Ok(())
}

#[test]
/// Test for load command if file doesn't exist and exit on error flag is enabled
fn load_file_dont_found_with_exit_on_error() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("envfetch")?;
    cmd.arg("delete").arg("MY_ENV_VAR_FOR_EXIT_WARNING_TEST");
    cmd.arg("--exit-on-warning");
    cmd.arg("echo hello").assert().failure();
    Ok(())
}

#[test]
/// Test for load command if custom file exist
fn load_custom_file_exists() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("envfetch")?;
    let file = assert_fs::NamedTempFile::new(".env.test")?;
    file.write_str("MY_ENV_VAR='TEST'\nTEST='hello'")?;
    cmd.arg("load").arg("--file").arg(file.path());
    // Windows
    #[cfg(target_os = "windows")]
    cmd.arg("echo %MY_ENV_VAR%")
        .assert()
        .success()
        .stdout(predicate::str::contains("TEST"));

    // Linux and macOS
    #[cfg(not(target_os = "windows"))]
    cmd.arg("echo $MY_ENV_VAR")
        .assert()
        .success()
        .stdout(predicate::str::contains("TEST"));
    // Close file after test
    file.close().unwrap();
    Ok(())
}

#[test]
/// Test for load command if custom file exist and specified process failed
fn load_custom_file_exists_command_failed() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("envfetch")?;
    let file = assert_fs::NamedTempFile::new(".env.test")?;
    file.assert(predicate::path::missing());
    file.write_str("MY_ENV_VAR='TEST'\nTEST='hello'")?;
    cmd.arg("load").arg("--file").arg(file.path());
    // Windows
    #[cfg(target_os = "windows")]
    cmd.arg("echo %MY_ENV_VAR_TEST%")
        .assert()
        .success()
        .stdout(predicate::str::contains("%MY_ENV_VAR_TEST%"));

    // Linux and macOS
    #[cfg(not(target_os = "windows"))]
    cmd.arg("(exit 1)").assert().failure();
    // Close file after test
    file.close().unwrap();
    Ok(())
}

#[test]
/// Test for load command if custom file doesn't exist
fn load_custom_file_doesnt_exists() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("envfetch")?;
    cmd.arg("load").arg("--file").arg(".env.production");
    // Windows
    #[cfg(target_os = "windows")]
    cmd.arg("echo %MY_ENV_VAR%").assert().failure();

    // Linux and macOS
    #[cfg(not(target_os = "windows"))]
    cmd.arg("echo $MY_VARIABLE").assert().failure();
    Ok(())
}

// TODO: add tests for commands with --global flag
