# Rust Rewrite & Otterscan Integration

## Architecture
1.  **Monolithic Binary (Rust):** Replace Bash + Python scripts with a single statically-compiled Rust executable.
2.  **Dependencies:**
    *   `clap` (CLI), `tokio` (async/subprocesses), `alloy` (RPC/types).
    *   `serde`/`serde_json` for genesis mutation.
    *   `anyhow`, `tracing`, `indicatif` for error handling and UI.
3.  **Process Management:**
    *   Strict child process oversight. When `forge-persist` is killed or panics, `anvil` and `reth` MUST automatically die to prevent port zombie locks. We will use proper POSIX process group signaling (`nix`).
4.  **Otterscan Explorer Integration:**
    *   **Proposed Alternative:** Do not embed the static React app in the Rust binary (causes severe binary bloat and difficult maintenance). Enable the `ots,eth,net,web3` namespaces in Reth via CLI flags, and advise users to point any standard web-hosted Otterscan instance to their `localhost:8545`.

## Phase 4 Completed
- The `src/convert.rs` engine is fully wired into `main.rs`.
- A two-step Reth boot pipeline utilizing `tokio::process::Command` now spins up the MDBX database natively (`reth init`) before hanging the thread on the Reth server process (`reth node`).
- A `ctrl_c` trap strictly intercepts exit signals, broadcasting SIGTERM to Reth to guarantee database durability before releasing the terminal.

## Phase 5 (Distribution & Documentation)
1. **Track A (cargo-dist):** Initialize `cargo-dist` to automatically generate `release.yml` targeting macOS Apple Silicon, macOS Intel, and Linux, provisioning a `curl` bash installer script natively.
2. **Track B (VitePress):** Scaffold a `/docs` directory specifically targeting Web3 developers suffering from Anvil memory leaks, utilizing a high-performance VitePress configuration and deploying automatically via GitHub Pages.

## Review Required
- Needs CEO approval to commence Phase 5 (`cargo-dist` deployment logic and `/docs` scaffolding) per the Stage-Gate Protocol.
