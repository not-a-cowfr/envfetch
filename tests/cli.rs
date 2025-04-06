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
    cmd.arg("--")
        .arg("echo %MY_VAR%")
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello"));

    // Linux and macOS
    #[cfg(not(target_os = "windows"))]
    cmd.arg("--")
        .arg("echo $MY_VAR")
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
    cmd.arg("--").arg("%MY_VARIABLE%").assert().failure();
    Ok(())
}

#[test]
/// Test for get command if specified variable exists
fn get_variable_exists() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("envfetch")?;
    unsafe { env::set_var("MY_VAR", "Hello") };
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
        .stderr(predicate::str::contains("Can't find variable: MY_VARIABLE"));
    Ok(())
}

#[test]
/// Test for get command if specified variable doesn't exist and showing similar variables is enabled
fn get_variable_doesnt_exists_similar_enabled() -> Result<(), Box<dyn std::error::Error>> {
    unsafe { env::set_var("MY_VARIABLEE", "Hello") };
    let mut cmd = Command::cargo_bin("envfetch")?;
    cmd.arg("get").arg("MY_VARIABLE");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Can't find variable: MY_VARIABLE"))
        .stdout(predicate::str::contains("Did you mean:"))
        .stdout(predicate::str::contains("MY_VARIABLEE"));
    Ok(())
}

#[test]
/// Test for get command if specified variable doesn't exist and showing similar variables is disabled
fn get_variable_doesnt_exists_similar_disabled() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("envfetch")?;
    unsafe { env::set_var("MY_VARIABLEE", "Hello") };
    cmd.arg("get").arg("MY_VARIABLE").arg("--no-similar-names");
    cmd.assert().failure();
    Ok(())
}

#[test]
/// Test for print command
fn print_success() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("envfetch")?;
    unsafe { env::set_var("PRINT_TEST", "Print") };
    cmd.arg("print")
        .assert()
        .success()
        .stdout(predicate::str::contains("PRINT_TEST = \"Print\""));
    Ok(())
}

#[test]
/// Test for print command
fn print_with_format_success() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("envfetch")?;
    unsafe { env::set_var("PRINT_TEST", "Print") };
    cmd.arg("print")
        .arg("--format")
        .arg("{name}: {value}")
        .assert()
        .success()
        .stdout(predicate::str::contains("PRINT_TEST: Print"));
    Ok(())
}

#[test]
/// Test for delete command if specified process is successful
fn delete_command_success() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("envfetch")?;
    unsafe { env::set_var("MY_VAR", "Hello") };
    cmd.arg("delete").arg("MY_VAR");
    // Windows
    #[cfg(target_os = "windows")]
    cmd.arg("--")
        .arg("echo 'Hello'")
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello"));

    // Linux and macOS
    #[cfg(not(target_os = "windows"))]
    cmd.arg("--")
        .arg("echo 'Hello'")
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello"));
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
    cmd.arg("--")
        .arg("echo %MY_ENV_VAR%")
        .assert()
        .success()
        .stdout(predicate::str::contains("TEST"));

    // Linux and macOS
    #[cfg(not(target_os = "windows"))]
    cmd.arg("--")
        .arg("echo $MY_ENV_VAR")
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
    cmd.arg("--")
        .arg("echo %MY_ENV_VAR_TEST%")
        .assert()
        .success()
        .stdout(predicate::str::contains("%MY_ENV_VAR_TEST%"));

    // Linux and macOS
    #[cfg(not(target_os = "windows"))]
    cmd.arg("--").arg("(exit 1)").assert().failure();
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
    cmd.arg("--").arg("echo %MY_ENV_VAR%").assert().failure();

    // Linux and macOS
    #[cfg(not(target_os = "windows"))]
    cmd.arg("--").arg("echo $MY_VARIABLE").assert().failure();
    Ok(())
}

#[test]
fn test_add_local_variable() -> Result<(), Box<dyn std::error::Error>> {
    let envfetch = Command::cargo_bin("envfetch")?
        .get_program()
        .to_string_lossy()
        .to_string();
    let mut cmd = Command::cargo_bin("envfetch")?;

    cmd.args([
        "add",
        "TEST_VAR",
        "test_value",
        "--",
        &format!("{} get TEST_VAR", envfetch),
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains("test_value"));

    Ok(())
}

#[test]
fn test_add_variable_with_special_characters() -> Result<(), Box<dyn std::error::Error>> {
    let envfetch = Command::cargo_bin("envfetch")?
        .get_program()
        .to_string_lossy()
        .to_string();
    let mut cmd = Command::cargo_bin("envfetch")?;

    cmd.args([
        "add",
        "SPECIAL_VAR",
        "test@#$%^&*",
        "--",
        &format!("{} get SPECIAL_VAR", envfetch),
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains("test@#$%^&*"));

    Ok(())
}

#[test]
fn test_add_empty_value() -> Result<(), Box<dyn std::error::Error>> {
    let envfetch = Command::cargo_bin("envfetch")?
        .get_program()
        .to_string_lossy()
        .to_string();
    let mut cmd = Command::cargo_bin("envfetch")?;

    cmd.args([
        "add",
        "EMPTY_VAR",
        "",
        "--",
        &format!("{} get EMPTY_VAR", envfetch),
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains("\"\"\n")); // Expect empty string in quotes with newline

    Ok(())
}

#[test]
fn test_add_invalid_variable_name() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("envfetch")?;
    cmd.args(["add", "INVALID NAME", "test_value", "--", "echo test"])
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "Variable name cannot contain spaces",
        ));
    Ok(())
}

#[test]
fn test_add_missing_value() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("envfetch")?;
    cmd.args(["add", "TEST_VAR"]).assert().failure();
    Ok(())
}

#[test]
fn test_add_with_process() -> Result<(), Box<dyn std::error::Error>> {
    let envfetch = Command::cargo_bin("envfetch")?
        .get_program()
        .to_string_lossy()
        .to_string();
    let mut cmd = Command::cargo_bin("envfetch")?;

    cmd.args([
        "add",
        "PROCESS_VAR",
        "test_value",
        "--",
        &format!("{} get PROCESS_VAR", envfetch),
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains("test_value"));

    Ok(())
}

#[test]
/// Test for print command with custom format
fn print_with_custom_format() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("envfetch")?;
    unsafe { env::set_var("FORMAT_TEST", "Hello") };
    cmd.arg("print")
        .arg("--format")
        .arg("{name}={value}")
        .assert()
        .success()
        .stdout(predicate::str::contains("FORMAT_TEST=Hello"));
    Ok(())
}
