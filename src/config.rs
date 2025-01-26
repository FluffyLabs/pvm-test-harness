use serde::Deserialize;
use std::{
    fs,
    path::{Path, PathBuf},
};

pub fn read_config_file(path: &Path) -> anyhow::Result<Config> {
    let config = fs::read_to_string(path)?;
    let config = toml::from_str(&config)?;
    Ok(config)
}

#[derive(Deserialize)]
pub struct Config {
    pub pvm: Vec<Pvm>,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(tag = "kind")]
#[serde(rename_all(deserialize = "lowercase"))]
pub enum Pvm {
    /// Built-in polkavm native interface.
    PolkaVM,

    /// stdin-based interface
    Stdin {
        name: Option<String>,
        binary: PathBuf,
    },

    #[allow(dead_code)]
    /// jsonrpc-based interface
    JsonRpc {
        name: Option<String>,
        endpoint: String,
    },
}

impl std::str::FromStr for Pvm {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "polkavm" {
            Ok(Pvm::PolkaVM)
        } else if s.starts_with("stdin=") {
            let path = std::path::PathBuf::from_str(s.trim_start_matches("stdin="))?;
            Ok(Pvm::Stdin {
                name: None,
                binary: path,
            })
        } else if s.starts_with("jsonrpc=") {
            Ok(Pvm::JsonRpc {
                name: None,
                endpoint: s.trim_start_matches("rpc=").to_string(),
            })
        } else {
            anyhow::bail!("Invalid PVM argument: {}", s)
        }
    }
}
