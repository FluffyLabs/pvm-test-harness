use std::path::PathBuf;
use clap::Parser;

mod api;

fn main() {
    env_logger::init();
    let args = Args::parse();

    match args {
        Args::Json { pvm } => {
            todo!();
        },
        Args::Fuzz { pvm } => {
            todo!();
        },
    }
}

#[derive(Parser, Debug)]
#[clap(version)]
/// Run test harness for PVMs.
enum Args {
    /// Execute a JSON test case.
    Json {
        #[clap(help=PVM_HELP)]
        pvm: Vec<Pvm>,
    },
    /// Run fuzz testing.
    Fuzz {
        #[clap(help=PVM_HELP)]
        pvm: Vec<Pvm>,
    }
}

const PVM_HELP: &str = "PVMs to run. Can be either 'polkavm', 'stdin=<path>' or jsonrpc=<endpoint>.";
#[derive(Debug, Clone)]
enum Pvm {
    /// Built-in polkavm native interface.
    PolkaVM,

    /// stdin-based interface
    Stdin {
        binary: PathBuf,
    },

    /// jsonrpc-based interface
    JsonRpc {
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
            Ok(Pvm::Stdin { binary: path })
        } else if s.starts_with("jsonrpc=") {
            Ok(Pvm::JsonRpc { endpoint: s.trim_start_matches("rpc=").to_string() })
        } else {
            anyhow::bail!("Invalid PVM argument: {}", s)
        }
    }
}
