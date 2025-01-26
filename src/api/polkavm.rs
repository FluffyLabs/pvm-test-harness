use super::{
    common::{InitialState, OutputState},
    Error, ProgramContainer, PvmApi, Status,
};

#[derive(Debug, Default)]
pub struct PolkaVm {
    initial: InitialState,
    output: OutputState,
}

impl PolkaVm {
    fn init_instance(&self) -> super::Result<polkavm::RawInstance> {
        let parts = match self.initial.container {
            Some(ProgramContainer::Generic) => {
                let mut parts = polkavm::ProgramParts::default();
                parts.code_and_jump_table = self.initial.program.clone().into();
                parts.is_64_bit = true;
                // TODO [ToDr] setup memory
                Ok(parts)
            }
            Some(ProgramContainer::PolkaVM) => polkavm::ProgramParts::from_bytes(
                self.initial.program.clone().into(),
            )
            .map_err(|e| {
                log::error!("{:?}", e);
                Error::InvalidProgram
            }),
            _ => Err(Error::UnsupportedContainer),
        }?;

        let blob = polkavm::ProgramBlob::from_parts(parts).map_err(|e| {
            log::error!("{:?}", e);
            Error::InvalidProgram
        })?;

        let mut config = polkavm::Config::new();
        config.set_backend(Some(polkavm::BackendKind::Interpreter));
        let engine = polkavm::Engine::new(&config).unwrap();

        let mut module_config = polkavm::ModuleConfig::default();
        module_config.set_strict(true);
        module_config.set_gas_metering(Some(polkavm::GasMeteringKind::Sync));
        //module_config.set_step_tracing(true);

        let module = polkavm::Module::from_blob(&engine, &module_config, blob).unwrap();
        let mut instance = module.instantiate().unwrap();

        instance.set_gas(self.initial.gas);
        instance.set_next_program_counter(polkavm::ProgramCounter(self.initial.pc));
        for (reg, v) in polkavm::Reg::ALL.iter().zip(self.initial.registers) {
            instance.set_reg(*reg, v);
        }

        Ok(instance)
    }
}

impl PvmApi for PolkaVm {
    fn run(&mut self) -> super::Result<Status> {
        log::debug!("[polkavm] executing: {:?}", self);
        use polkavm::InterruptKind::*;

        let mut instance = self.init_instance()?;
        let status = instance.run();
        let status = match status {
            Ok(Finished) => Status::Halt,
            Ok(Trap) => Status::Trap,
            Ok(Ecalli(_call)) => Status::Host,
            Ok(Segfault(_page)) => Status::Fault,
            Ok(NotEnoughGas) => Status::OutOfGas,
            Ok(Step) => Status::Ok,
            Err(e) => {
                log::error!("Error: {:?}", e);
                Status::Trap
            }
        };

        // copy results to `output` so they can be queried
        self.output.gas = instance.gas();
        self.output.pc = instance.program_counter().map(|x| x.0);
        for (reg, out) in polkavm::Reg::ALL.iter().zip(&mut self.output.registers) {
            *out = instance.reg(*reg);
        }

        log::debug!("[polkavm] Complete with status {status}: {:?}", self.output);
        Ok(status)
    }

    fn gas(&self) -> i64 {
        self.output.gas
    }

    fn set_gas(&mut self, gas: i64) {
        self.initial.gas = gas;
    }

    fn registers(&self) -> [u64; super::NUMBER_OF_REGISTERS] {
        self.output.registers
    }

    fn set_registers(&mut self, registers: &[u64; super::NUMBER_OF_REGISTERS]) {
        self.initial.registers = registers.to_owned();
    }

    fn program_counter(&self) -> Option<u32> {
        self.output.pc
    }

    fn set_next_program_counter(&mut self, pc: u32) {
        self.initial.pc = pc;
    }

    fn set_program(
        &mut self,
        code: &[u8],
        container: super::ProgramContainer,
    ) -> super::Result<()> {
        if let ProgramContainer::Spi = container {
            return Err(Error::UnsupportedContainer);
        }
        // TODO [ToDr] shall we parse the program here already?
        self.initial.program = code.to_vec();
        self.initial.container = Some(container);
        Ok(())
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
