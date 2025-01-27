use anyhow::Context;
use api::PvmApi;
use clap::{Parser, Subcommand};
use config::{read_config_file, Pvm};
use json::TestcaseJson;
use std::{path::PathBuf, process::Stdio};

mod api;
mod config;
mod json;

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let args = Args::parse();

    match args.sub {
        Command::Json { files } => {
            let pvm = with_config(args.config, args.pvm)?;
            // intialize pvms
            let mut pvms = api::collection::PvmApiCollection::new(init_pvms(&pvm)?);

            for file in files {
                let json = std::fs::read(&file).with_context(|| format!("Failed to read JSON file."))?;
                let json: TestcaseJson =
                    serde_json::from_slice(&json).with_context(|| format!("Failed to parse JSON file."))?;

                println!("{} running on {} pvms...", json.name, pvm.len());
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

                println!("{} ✅", json.name);
            }
            Ok(())
        }
        Command::Fuzz { .. } => {
            todo!();
        }
    }
}

fn init_pvms(pvm: &[Pvm]) -> anyhow::Result<Vec<Box<dyn PvmApi>>> {
    if pvm.is_empty() {
        anyhow::bail!("No PVMs specified. Make sure to start at least one.");
    }

    pvm.iter()
        .map(|pvm| {
            match pvm {
                Pvm::PolkaVM => Ok(Box::new(api::polkavm::PolkaVm::default()) as Box<dyn PvmApi>),
                Pvm::Stdin { name, binary } => {
                    // spawn process
                    let process = std::process::Command::new(binary)
                        .stdin(Stdio::piped())
                        .stdout(Stdio::piped())
                        .stderr(Stdio::inherit())
                        .spawn()
                        .with_context(|| format!("Unable to start stdin pvm: {name:?}"))?;
                    let stdin = process.stdin.unwrap();
                    let stdout = process.stdout.unwrap();
                    Ok(Box::new(api::stdin::JsonStdin::new(stdout, stdin)) as _)
                }
                Pvm::JsonRpc { .. } => {
                    anyhow::bail!("RPC pvm is not supported yet.")
                }
            }
        })
        .collect()
}

fn with_config(config: Option<PathBuf>, mut pvms: Vec<Pvm>) -> anyhow::Result<Vec<Pvm>> {
    match config {
        Some(path) => {
            let mut config = read_config_file(&path).with_context(|| format!("Failed to read the config file."))?;
            pvms.append(&mut config.pvm);
            Ok(pvms)
        }
        None => Ok(pvms),
    }
}

/// Run test harness for PVMs.
#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// toml config file.
    #[arg(short, long)]
    config: Option<PathBuf>,
    #[clap(help=PVM_HELP)]
    #[arg(long)]
    pvm: Vec<Pvm>,
    /// command to execute
    #[command(subcommand)]
    sub: Command,
}

/// Run test harness for PVMs.
#[derive(Subcommand, Debug, Clone)]
enum Command {
    /// Execute a JSON test case.
    Json {
        /// JSON file to load
        files: Vec<PathBuf>,
    },
    /// Run fuzz testing.
    Fuzz {},
}

const PVM_HELP: &str = "PVMs to run. Can be either 'polkavm', 'stdin=<path>' or jsonrpc=<endpoint>.";
