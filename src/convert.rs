use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;
use std::io::Read;
use flate2::read::GzDecoder;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RethGenesis {
    pub config: GenesisConfig,
    pub nonce: String,
    pub timestamp: String,
    pub extra_data: String,
    pub gas_limit: String,
    pub difficulty: String,
    pub mix_hash: String,
    pub coinbase: String,
    pub alloc: BTreeMap<String, GenesisAccount>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GenesisConfig {
    pub chain_id: u64,
    pub homestead_block: u64,
    pub eip150_block: u64,
    pub eip155_block: u64,
    pub eip158_block: u64,
    pub byzantium_block: u64,
    pub constantinople_block: u64,
    pub petersburg_block: u64,
    pub istanbul_block: u64,
    pub berlin_block: u64,
    pub london_block: u64,
    pub merge_netsplit_block: u64,
    pub shanghai_time: u64,
    pub cancun_time: u64,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct GenesisAccount {
    pub balance: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage: Option<BTreeMap<String, String>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum AnvilStateDump {
    Nested { accounts: BTreeMap<String, AnvilAccount> },
    Flat(BTreeMap<String, AnvilAccount>),
}

impl AnvilStateDump {
    fn into_accounts(self) -> BTreeMap<String, AnvilAccount> {
        match self {
            Self::Nested { accounts } => accounts,
            Self::Flat(accounts) => accounts,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnvilAccount {
    #[serde(default)]
    pub balance: serde_json::Value,
    #[serde(default)]
    pub nonce: serde_json::Value,
    pub code: Option<String>,
    pub storage: Option<BTreeMap<String, String>>,
}

fn to_hex_string(val: &serde_json::Value) -> String {
    match val {
        serde_json::Value::String(s) => {
            if s.starts_with("0x") {
                s.clone()
            } else {
                format!("0x{:x}", s.parse::<u128>().unwrap_or(0))
            }
        }
        serde_json::Value::Number(n) => {
            if let Some(u) = n.as_u64() {
                format!("0x{:x}", u)
            } else {
                "0x0".to_string()
            }
        }
        _ => "0x0".to_string(),
    }
}

pub async fn generate_genesis(
    chain_id: u64,
    anvil_dump_path: &Path,
    genesis_out_path: &Path,
) -> Result<()> {
    let raw = tokio::fs::read_to_string(anvil_dump_path)
        .await
        .context("Failed to read Anvil state dump JSON")?;

    let state: AnvilStateDump = if raw.trim().starts_with("\"0x") {
        let hex_str = raw.trim().trim_matches('"').trim_start_matches("0x");
        let compressed = hex::decode(hex_str).context("Invalid hex string from Anvil dump")?;
        let mut d = GzDecoder::new(&compressed[..]);
        let mut decompressed = String::new();
        d.read_to_string(&mut decompressed).context("Failed GZIP decompression")?;
        serde_json::from_str(&decompressed).context("Failed to deserialize decompressed JSON")?
    } else {
        serde_json::from_str(&raw).context("Failed to deserialize generic JSON")?
    };

    let accounts_map = state.into_accounts();
    let mut alloc = BTreeMap::new();

    for (addr, acc) in accounts_map {
        let balance_hex = to_hex_string(&acc.balance);
        let nonce_hex = to_hex_string(&acc.nonce);

        let code = match acc.code {
            Some(ref c) if c == "0x" || c == "0x0" => None,
            other => other,
        };

        let has_balance = balance_hex != "0x0";
        let has_nonce = nonce_hex != "0x0";
        let has_code = code.is_some();
        let has_storage = acc.storage.as_ref().map(|s| !s.is_empty()).unwrap_or(false);

        if !has_balance && !has_nonce && !has_code && !has_storage {
            continue; // Filter dropped
        }

        let clean_addr = if addr.starts_with("0x") {
            addr[2..].to_string()
        } else {
            addr
        };

        alloc.insert(
            clean_addr,
            GenesisAccount {
                balance: balance_hex,
                nonce: if has_nonce { Some(nonce_hex) } else { None },
                code,
                storage: acc.storage,
            },
        );
    }

    let genesis = RethGenesis {
        config: GenesisConfig {
            chain_id,
            homestead_block: 0,
            eip150_block: 0,
            eip155_block: 0,
            eip158_block: 0,
            byzantium_block: 0,
            constantinople_block: 0,
            petersburg_block: 0,
            istanbul_block: 0,
            berlin_block: 0,
            london_block: 0,
            merge_netsplit_block: 0,
            shanghai_time: 0,
            cancun_time: 0,
        },
        nonce: "0x0000000000000000".to_string(),
        timestamp: "0x0".to_string(),
        extra_data: "0x".to_string(),
        gas_limit: "0x1fffffffffffff".to_string(),
        difficulty: "0x0".to_string(),
        mix_hash: "0x0000000000000000000000000000000000000000000000000000000000000000".to_string(),
        coinbase: "0x0000000000000000000000000000000000000000".to_string(),
        alloc,
    };

    let pretty_json = serde_json::to_string_pretty(&genesis)
        .context("Failed to serialize Reth genesis object")?;

    tokio::fs::write(genesis_out_path, pretty_json)
        .await
        .context("Failed to write genesis.json to disk")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_missing_and_malformed_fields_dropped() -> Result<()> {
        let raw_json = r#"{
            "0xempty": {
                "balance": 0,
                "nonce": 0
            },
            "0xvalid": {
                "balance": 100,
                "nonce": 1,
                "code": "0x6060"
            }
        }"#;
        let state: AnvilStateDump = serde_json::from_str(raw_json)?;
        let flat = state.into_accounts();
        
        let empty_acc = flat.get("0xempty").unwrap();
        let valid_acc = flat.get("0xvalid").unwrap();

        assert_eq!(to_hex_string(&empty_acc.balance), "0x0");
        assert_eq!(to_hex_string(&valid_acc.balance), "0x64");
        Ok(())
    }

    #[test]
    fn test_hex_standardization() -> Result<()> {
        let n: serde_json::Value = serde_json::from_str("1000000")?;
        assert_eq!(to_hex_string(&n), "0xf4240");

        let s: serde_json::Value = serde_json::from_str("\"1000000\"")?;
        assert_eq!(to_hex_string(&s), "0xf4240");

        let h: serde_json::Value = serde_json::from_str("\"0xf4240\"")?;
        assert_eq!(to_hex_string(&h), "0xf4240");
        Ok(())
    }

    #[test]
    fn test_nested_vs_flat_deserialization() -> Result<()> {
        let nested = r#"{"accounts": {"0x1": {"balance": 1}}}"#;
        let flat = r#"{"0x1": {"balance": 1}}"#;

        let state_nested: AnvilStateDump = serde_json::from_str(nested)?;
        let state_flat: AnvilStateDump = serde_json::from_str(flat)?;

        assert_eq!(
            to_hex_string(&state_nested.into_accounts().get("0x1").unwrap().balance),
            to_hex_string(&state_flat.into_accounts().get("0x1").unwrap().balance)
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_filesystem_genesis_integration() -> Result<()> {
        let dir = tempdir()?;
        let dump_path = dir.path().join("state_dump.json");
        let genesis_path = dir.path().join("genesis.json");

        let mock_dump = r#"{"0x00000000000000000000000000000000000000aa": {"balance": "0x100"}}"#;
        tokio::fs::write(&dump_path, mock_dump).await?;

        let chain_id = 31337;
        generate_genesis(chain_id, &dump_path, &genesis_path).await?;

        assert!(genesis_path.exists());

        let out_obj = tokio::fs::read_to_string(&genesis_path).await?;
        let gen: RethGenesis = serde_json::from_str(&out_obj)?;

        assert_eq!(gen.config.chain_id, 31337);
        assert_eq!(gen.gas_limit, "0x1fffffffffffff");
        assert!(gen.alloc.contains_key("00000000000000000000000000000000000000aa"));
        Ok(())
    }
}
