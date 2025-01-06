use std::{path::PathBuf, process::Stdio};
use api::PvmApi;
use clap::Parser;
use json::TestcaseJson;

mod api;
mod json;

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let args = Args::parse();

    match args {
        Args::Json { file, pvm } => {
            let json = std::fs::read(&file)?;
            let json: TestcaseJson = serde_json::from_slice(&json)?;

            // intialize pvms
            let mut pvms = api::collection::PvmApiCollection::new(init_pvms(&pvm)?);

            let mut registers = [0u64; api::NUMBER_OF_REGISTERS];
            for (out, reg) in &mut registers.iter_mut().zip(&json.initial_regs) {
                *out = *reg;
            }
            pvms.set_gas(json.initial_gas);
            pvms.set_registers(&registers);
            pvms.set_next_program_counter(json.initial_pc);
            // TODO [ToDr] setup memory?
            pvms.set_program(&json.program, api::ProgramContainer::Generic)?;

            let status = pvms.run()?;
            let regs = pvms.registers();
            let gas = pvms.gas();
            let pc = pvms.program_counter();

            assert_eq!(format!("{status}"), json.expected_status, "Mismatching status");
            assert_eq!(gas, json.expected_gas, "Mismatching gas");
            assert_eq!(pc, Some(json.expected_pc), "Mismatching pc");
            assert_eq!(&regs, &*json.expected_regs, "Mismatching regs");
            // TODO [ToDr] Compare memory
            
            println!("{} executed", json.name);
            Ok(())
        },
        Args::Fuzz { .. } => {
            todo!();
        },
    }
}

fn init_pvms(pvm: &[Pvm]) -> anyhow::Result<Vec<Box<dyn PvmApi>>> {
    if pvm.is_empty() {
        anyhow::bail!("No PVMs specified. Make sure to start at least one.");
    }

    pvm.iter().map(|pvm| {
        match pvm {
            Pvm::PolkaVM => Ok(Box::new(api::polkavm::PolkaVm::default()) as Box<dyn PvmApi>),
            Pvm::Stdin { binary } => {
                // spawn process
                let process = std::process::Command::new(&binary)
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::inherit())
                    .spawn()?;
                let stdin = process.stdin.unwrap();
                let stdout = process.stdout.unwrap();
                Ok(Box::new(api::stdin::JsonStdin::new(stdout, stdin)) as _)
            },
            Pvm::JsonRpc { .. } => {
                anyhow::bail!("RPC pvm is not supported yet.")
            }
        }
    }).collect()
}

#[derive(Parser, Debug)]
#[clap(version)]
/// Run test harness for PVMs.
enum Args {
    /// Execute a JSON test case.
    Json {
        file: PathBuf,
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

    #[allow(dead_code)]
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
