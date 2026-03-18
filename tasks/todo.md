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

## Phase 5 Completed
- Integrated automatic Apple Silicon and Linux cross-compilation matrix using `cargo-dist`.
- Rendered GitHub continuous-deployment workflows protecting the binary distribution payload.
- Finalized VitePress `/docs` platform deployments leveraging targeted push branches securely.

## Phase 6 Completed
- Scaffolded physical unit and integration tests under `tempfile` constraints successfully verifying translation isolation.
- Extracted strict dependency logic testing into `tests/cli.rs` ensuring robust black-box `assert_cmd` bounds catch UI exceptions natively inside GitHub Actions.

## Phase 7 Completed
- Implemented `script/StressTest.s.sol` bounded precisely to 1,000 distinct transactions to strictly assert rapid mempool exhaustion natively.
- Wrote `bin/monitor-mem.sh` enforcing rigorous `lsof -ti:8545` port locks to cleanly extract node telemetry bounding exactly to the local RPC.
- Constructed `benchmark.md` runbook explicitly mapping the side-by-side split screen verification boundaries sequentially.

## Final Review
- The repository logic, cross-platform generation scripts, and internal benchmarking engines are completely aligned. The architectural lifecycle is absolutely finished.

## Phase 8 (Bugfix: Foundry Init & Custom Ports)
- Generated `foundry.toml` and natively pinned `forge-std` without committing the repository to securely compile `StressTest.s.sol`.
- Refactored `benchmark.md` to structurally map Anvil telemetry tests precisely to `--port 4000` and `forge-persist` securely to `--port 4001`.
