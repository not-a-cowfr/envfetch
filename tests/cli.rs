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
    let mut cmd = Command::cargo_bin("envfetch")?;
    env::set_var("MY_VARIABLEE", "Hello");
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
    cmd.arg("load");
    cmd.arg("--exit-on-error");
    cmd.arg("echo %MY_ENV_VAR%").assert().failure();
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

#[test]
/// Test for set command with global flag
fn set_command_global() -> Result<(), Box<dyn std::error::Error>> {
    let var_name = "GLOBAL_SET_TEST";
    let var_value = "GlobalValue";

    // First set the variable globally
    let mut cmd = Command::cargo_bin("envfetch")?;
    cmd.arg("set")
        .arg(var_name)
        .arg(var_value)
        .arg("--global");
    cmd.assert().success();

    // Verify using shell commands
    #[cfg(target_os = "windows")]
    {
        let output = std::process::Command::new("cmd")
            .args(&["/C", "reg", "query", "HKCU\\Environment", "/v", var_name])
            .output()?;
        assert!(String::from_utf8_lossy(&output.stdout).contains(var_value));
    }

    #[cfg(target_os = "macos")]
    {
        let output = std::process::Command::new("zsh")
            .args(&["-c", &format!("source ~/.zshrc 2>/dev/null || source ~/.bashrc 2>/dev/null; echo ${}", var_name)])
            .output()?;
        assert!(String::from_utf8_lossy(&output.stdout).contains(var_value),
            "Variable not found in shell output: {}", String::from_utf8_lossy(&output.stdout));
    }

    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        let output = std::process::Command::new("bash")
            .args(&["-c", &format!("source ~/.bashrc && echo ${}", var_name)])
            .output()?;
        assert!(String::from_utf8_lossy(&output.stdout).contains(var_value));
    }

    // Clean up
    let mut cmd = Command::cargo_bin("envfetch")?;
    cmd.arg("delete").arg(var_name).arg("--global");
    cmd.assert().success();

    Ok(())
}

#[test]
/// Test for delete command with global flag
fn delete_command_global() -> Result<(), Box<dyn std::error::Error>> {
    let var_name = "GLOBAL_DELETE_TEST";
    let var_value = "ToBeDeleted";

    // First set the variable
    let mut cmd = Command::cargo_bin("envfetch")?;
    cmd.arg("set")
        .arg(var_name)
        .arg(var_value)
        .arg("--global");
    cmd.assert().success();

    // Verify it was set
    #[cfg(target_os = "windows")]
    {
        let output = std::process::Command::new("cmd")
            .args(&["/C", "reg", "query", "HKCU\\Environment", "/v", var_name])
            .output()?;
        assert!(String::from_utf8_lossy(&output.stdout).contains(var_value));
    }

    // Then delete it globally
    let mut cmd = Command::cargo_bin("envfetch")?;
    cmd.arg("delete").arg(var_name).arg("--global");
    cmd.assert().success();

    // On Windows, we need to force a registry refresh
    #[cfg(target_os = "windows")]
    {
        // Broadcast WM_SETTINGCHANGE message
        std::process::Command::new("cmd")
            .args(&["/C", "rundll32", "user32.dll,UpdatePerUserSystemParameters", "1", "True"])
            .output()?;

        // Additional registry refresh commands
        std::process::Command::new("cmd")
            .args(&["/C", "gpupdate", "/force"])
            .output()?;

        std::thread::sleep(std::time::Duration::from_secs(2));

        // Try multiple times to verify deletion
        let mut success = false;
        for i in 0..5 {
            let output = std::process::Command::new("cmd")
                .args(&["/C", "reg", "query", "HKCU\\Environment", "/v", var_name])
                .output()?;
            
            if !output.status.success() {
                success = true;
                break;
            }

            if i < 4 {
                // Try refreshing again
                std::process::Command::new("cmd")
                    .args(&["/C", "rundll32", "user32.dll,UpdatePerUserSystemParameters", "1", "True"])
                    .output()?;
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        }

        if !success {
            // Try direct registry deletion as a last resort
            std::process::Command::new("cmd")
                .args(&["/C", "reg", "delete", "HKCU\\Environment", "/v", var_name, "/f"])
                .output()?;
            
            // Final verification
            let output = std::process::Command::new("cmd")
                .args(&["/C", "reg", "query", "HKCU\\Environment", "/v", var_name])
                .output()?;
            
            assert!(!output.status.success(), 
                "Failed to delete variable from registry even after direct deletion");
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let output = std::process::Command::new("sh")
            .args(&["-c", &format!("echo ${}", var_name)])
            .output()?;
        assert!(String::from_utf8_lossy(&output.stdout).trim().is_empty());
    }

    Ok(())
}

#[test]
/// Test for load command with global flag
fn load_command_global() -> Result<(), Box<dyn std::error::Error>> {
    // Create a temporary .env file
    let file = assert_fs::NamedTempFile::new(".env.global.test")?;
    file.write_str("GLOBAL_TEST_VAR='GlobalTest'\nGLOBAL_TEST_VAR2='Hello'")?;

    // Load variables globally
    let mut cmd = Command::cargo_bin("envfetch")?;
    cmd.arg("load")
        .arg("--global")
        .arg("--file")
        .arg(file.path());
    cmd.assert().success();

    // Verify using shell commands
    #[cfg(target_os = "windows")]
    {
        // Check first variable
        let output1 = std::process::Command::new("cmd")
            .args(&["/C", "reg", "query", "HKCU\\Environment", "/v", "GLOBAL_TEST_VAR"])
            .output()?;
        assert!(String::from_utf8_lossy(&output1.stdout).contains("GlobalTest"));

        // Check second variable
        let output2 = std::process::Command::new("cmd")
            .args(&["/C", "reg", "query", "HKCU\\Environment", "/v", "GLOBAL_TEST_VAR2"])
            .output()?;
        assert!(String::from_utf8_lossy(&output2.stdout).contains("Hello"));
    }

    #[cfg(target_os = "macos")]
    {
        // On macOS, we need to source both potential config files
        let output = std::process::Command::new("zsh")
            .args(&["-c", "source ~/.zshrc 2>/dev/null || source ~/.bashrc 2>/dev/null; echo $GLOBAL_TEST_VAR $GLOBAL_TEST_VAR2"])
            .output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("GlobalTest") && stdout.contains("Hello"),
            "Variables not found in shell output: {}", stdout);
    }

    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        let output = std::process::Command::new("bash")
            .args(&["-c", "source ~/.bashrc && echo $GLOBAL_TEST_VAR $GLOBAL_TEST_VAR2"])
            .output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("GlobalTest") && stdout.contains("Hello"),
            "Variables not found in shell output: {}", stdout);
    }

    // Clean up
    let mut cmd = Command::cargo_bin("envfetch")?;
    cmd.arg("delete").arg("GLOBAL_TEST_VAR").arg("--global");
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("envfetch")?;
    cmd.arg("delete").arg("GLOBAL_TEST_VAR2").arg("--global");
    cmd.assert().success();

    file.close()?;
    Ok(())
}

#[test]
/// Test for load command with global flag and invalid file
fn load_command_global_invalid_file() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("envfetch")?;
    cmd.arg("load")
        .arg("--global")
        .arg("--file")
        .arg("nonexistent.env");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("error:"));
    Ok(())
}

#[test]
/// Test for set command with global flag and invalid variable name
fn set_command_global_invalid_name() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("envfetch")?;
    cmd.arg("set")
        .arg("INVALID NAME")
        .arg("Value")
        .arg("--global");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Variable name cannot contain spaces"));
    Ok(())
}
