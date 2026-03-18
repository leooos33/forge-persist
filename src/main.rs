mod convert;

use alloy::providers::{Provider, ProviderBuilder};
use anyhow::{Context, Result};
use clap::Parser;
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
use std::time::Duration;
use tokio::process::Command;
use url::Url;

#[derive(Parser, Debug)]
#[command(name = "forge-persist", version, about = "Persistent mainnet forks in one command.")]
struct Cli {
    #[arg(long)]
    fork_url: Option<String>,

    #[arg(long)]
    fork_block: Option<u64>,

    #[arg(long)]
    deploy: Option<String>,

    #[arg(long, default_value_t = 8545)]
    port: u16,

    #[arg(long, default_value_t = 31337)]
    chain_id: u64,

    #[arg(long, default_value = ".forge-persist")]
    data_dir: String,

    #[arg(long)]
    resume: bool,

    #[arg(long)]
    explorer: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    // Enforce POSIX process group logic
    unsafe {
        libc::setpgid(0, 0);
    }

    let data_dir = std::path::Path::new(&cli.data_dir);
    tokio::fs::create_dir_all(data_dir)
        .await
        .context("Failed to create data directory")?;

    let dump_path = data_dir.join("state_dump.json");
    let genesis_path = data_dir.join("genesis.json");
    let reth_db_path = data_dir.join("reth-db");

    if cli.resume {
        boot_reth(&genesis_path, &reth_db_path, cli.port, cli.explorer).await?;
        return Ok(());
    }

    let fork_url = cli
        .fork_url
        .as_ref()
        .context("Missing --fork-url. Required for first run.")?;

    println!("{}", "══════════════════════════════════════════════".blue());
    println!("{}", " forge-persist — Booting Persistent Fork".blue().bold());
    println!("{}", "══════════════════════════════════════════════\n".blue());

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{spinner:.green} {msg}")?,
    );
    pb.enable_steady_tick(Duration::from_millis(100));

    // Step 1: Initialize the Alloy HTTP Provider & Boot Temp Anvil
    let rpc_url: Url = "http://localhost:8546"
        .parse()
        .context("Failed to parse localhost RPC URL")?;
    let provider = ProviderBuilder::new().on_http(rpc_url);

    pb.set_message("Booting temp Anvil fork...");
    let mut anvil = Command::new("anvil")
        .arg("--fork-url")
        .arg(fork_url)
        .arg("--port")
        .arg("8546")
        .spawn()
        .context("Failed to spawn Anvil. Is Foundry installed?")?;

    // Step 2: Implement the Healthcheck (Polling Loop)
    let mut ready = false;
    for _ in 0..100 {
        if provider.get_block_number().await.is_ok() {
            ready = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    if !ready {
        if let Some(id) = anvil.id() {
            let _ = signal::kill(Pid::from_raw(id as i32), Signal::SIGKILL);
        }
        anyhow::bail!("Anvil failed to boot within 30 seconds. Port 8546 may be bound.");
    }
    pb.set_message("Anvil ready.");

    // Step 3: Execute Deployment (Optional)
    if let Some(deploy_cmd) = &cli.deploy {
        pb.set_message(format!("Running deploy script: {}", deploy_cmd));
        let mut deploy = Command::new("sh")
            .arg("-c")
            .arg(deploy_cmd)
            .spawn()
            .context("Failed to spawn deploy script wrapper")?;
        
        let status = deploy.wait().await?;
        if !status.success() {
            if let Some(id) = anvil.id() {
                let _ = signal::kill(Pid::from_raw(id as i32), Signal::SIGTERM);
                let _ = anvil.wait().await;
            }
            anyhow::bail!("Deploy script failed to execute successfully.");
        }
    }

    // Step 4: Implement the anvil_dumpState Call
    pb.set_message("Dumping local Anvil state via Alloy...");
    
    let dump: serde_json::Value = provider
        .client()
        .request("anvil_dumpState", ())
        .await
        .context("Failed to call anvil_dumpState via RPC")?;

    let dump_str = serde_json::to_string(&dump).context("Failed to serialize dump payload")?;
    tokio::fs::write(&dump_path, dump_str)
        .await
        .context("Failed to write state_dump.json to disk")?;

    pb.set_message("State dumped to disk.");

    // Step 5: Graceful Shutdown Updates
    pb.set_message("Gracefully terminating Anvil...");
    if let Some(id) = anvil.id() {
        let _ = signal::kill(Pid::from_raw(id as i32), Signal::SIGTERM);
    }
    let _ = anvil.wait().await;
    pb.set_message("Anvil terminated & Port 8546 released.");

    // Step 6: State Conversion & Boot Reth
    pb.set_message("Converting payload to Reth genesis...");
    convert::generate_genesis(cli.chain_id, &dump_path, &genesis_path)
        .await
        .context("Failed to generate Genesis object from Anvil dump")?;

    pb.finish_with_message("Genesis pipeline completed!");
    boot_reth(&genesis_path, &reth_db_path, cli.port, cli.explorer).await?;

    Ok(())
}

async fn boot_reth(genesis_path: &std::path::Path, reth_db_path: &std::path::Path, port: u16, explorer: bool) -> Result<()> {
    println!("\n{}", "🚀 Booting Reth Persistent Node...".green().bold());
    
    // The Init Step
    let mut init = Command::new("reth")
        .arg("init")
        .arg("--datadir")
        .arg(reth_db_path)
        .arg("--chain")
        .arg(genesis_path)
        .spawn()
        .context("Failed to spawn reth init. Is Reth installed in $PATH?")?;

    let init_status = init.wait().await?;
    if !init_status.success() {
        anyhow::bail!("Reth init failed. Genesis configuration may be invalid or corrupted.");
    }

    // The Node Step
    let mut node = Command::new("reth")
        .arg("node")
        .arg("--datadir")
        .arg(reth_db_path)
        .arg("--chain")
        .arg(genesis_path)
        .arg("--dev")
        .arg("--dev.block-time")
        .arg("1s")
        .arg("--http")
        .arg("--http.port")
        .arg(port.to_string())
        .arg("--http.api")
        .arg("eth,net,web3,debug,trace,ots")
        .arg("--http.corsdomain")
        .arg("*")
        .spawn()
        .context("Failed to spawn reth node process")?;

    println!("\n✅ {}", "Mainnet fork persistent and running!".green().bold());
    println!("🔗 RPC URL: http://localhost:{}", port);
    if explorer {
        println!("🔎 Explorer config: point standard Otterscan UI to port {}", port);
    }

    // Graceful Exit tracking
    tokio::signal::ctrl_c().await.context("Failed to listen for ctrl_c")?;
    println!("\n{}", "🛑 Shutting down Reth gracefully...".yellow());

    if let Some(id) = node.id() {
        let _ = signal::kill(Pid::from_raw(id as i32), Signal::SIGTERM);
    }
    let _ = node.wait().await;
    println!("{}", "✓ Reth shutdown complete. MDBX database intact.".green());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_cli_poka_yoke() {
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }
}
