use std::io::{BufRead, BufReader};

use crate::json::TestcaseJson;

use super::{common::OutputState, ProgramContainer, PvmApi, Status};

#[derive(Debug)]
pub struct JsonStdin<Read, Write> {
    json: TestcaseJson,
    output: OutputState,
    stdin: Write,
    stdout: Read,
}

impl<Read, Write> JsonStdin<Read, Write> {
    pub fn new(stdout: Read, stdin: Write) -> Self {
        Self {
            json: Default::default(),
            output: Default::default(),
            stdin,
            stdout,
        }
    }
}

impl<Read: std::io::Read, Write: std::io::Write> PvmApi for JsonStdin<Read, Write> {
    fn run(&mut self) -> super::Result<Status> {
        log::debug!("[stdin] Executing: {:?}", self.json);

        let read = BufReader::new(&mut self.stdout);
        let json = serde_json::to_vec(&self.json).unwrap();

        self.stdin.write_all(&json).map_err(super::Error::wrap)?;
        self.stdin.write_all(b"\n\n").map_err(super::Error::wrap)?;

        // read results
        let mut buffer = String::new();
        for line in read.lines() {
            let line = line.map_err(super::Error::wrap)?;
            // break on an empty line
            if line.is_empty() {
                break;
            }
            buffer.push_str(&line);
        }

        // copy results
        let output: TestcaseJson = serde_json::from_str(&buffer).map_err(super::Error::wrap)?;
        self.output.gas = output.expected_gas;
        self.output.pc = Some(output.expected_pc);
        for (out, reg) in self.output.registers.iter_mut().zip(&output.expected_regs) {
            *out = *reg;
        }

        let status = match &*output.expected_status {
            "trap" => Status::Trap,
            "out-of-gas" => Status::OutOfGas,
            "halt" => Status::Halt,
            "fault" => Status::Fault,
            "host" => Status::Host,
            _ => {
                log::error!("Invalid output status {}", output.expected_status);
                Status::Trap
            }
        };

        log::debug!("[stdin] Complete with status {status}: {:?}", self.output);
        Ok(status)
    }

    fn gas(&self) -> i64 {
        self.output.gas
    }

    fn set_gas(&mut self, gas: i64) {
        self.json.initial_gas = gas;
    }

    fn registers(&self) -> [u64; super::NUMBER_OF_REGISTERS] {
        self.output.registers
    }

    fn set_registers(&mut self, registers: &[u64; super::NUMBER_OF_REGISTERS]) {
        self.json.initial_regs = registers.to_owned();
    }

    fn program_counter(&self) -> Option<u32> {
        self.output.pc
    }

    fn set_next_program_counter(&mut self, pc: u32) {
        self.json.initial_pc = pc;
    }

    fn set_program(
        &mut self,
        code: &[u8],
        container: super::ProgramContainer,
    ) -> super::Result<()> {
        if let ProgramContainer::Generic = container {
            self.json.program = code.to_vec();
            Ok(())
        } else {
            Err(super::Error::Other("Unsupported container format.".into()))
        }
    }

    fn set_page(&mut self, _page: u32, _access: super::MemoryAccess) {
        todo!()
    }

    fn read_memory(&self, _address: u32, _out: &mut [u8]) -> super::Result<()> {
        todo!()
    }

    fn write_memory(&mut self, _address: u32, _data: &[u8]) -> super::Result<()> {
        todo!()
    }
}
