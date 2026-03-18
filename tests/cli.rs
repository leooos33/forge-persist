use anyhow::Result;
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_cli_help() -> Result<()> {
    let mut cmd = Command::cargo_bin("forge-persist")?;
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Persistent mainnet forks"));
    Ok(())
}

#[test]
fn test_missing_rpc_fallback() -> Result<()> {
    // Strictly isolate the black-box argument execution without spawning actual Anvil nodes!
    // We expect the native `anyhow::bail!` error wrapping to catch this error cleanly, rather than a Rust panic.
    let mut cmd = Command::cargo_bin("forge-persist")?;
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Missing --fork-url"));
    Ok(())
}
