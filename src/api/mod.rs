
mod common;
mod polkavm;
mod stdin;

const NUMBER_OF_REGISTERS: usize = 13;

#[derive(Copy, Clone, Debug)]
pub enum Status {
    Ok = 255,
    Halt = 0,
    Trap = 1,
    Fault = 2,
    Host = 3,
    OutOfGas = 4,
}

#[derive(Copy, Clone, Debug)]
pub enum ProgramContainer {
    Generic,
    SPI,
    PolkaVM,
}

#[derive(Copy, Clone, Debug)]
pub enum MemoryAccess {
    Readable,
    Writeable,
}

/// Low-level PVM interface.
pub trait PvmApi {
    type Error;

    fn run(&mut self) -> Result<Status, Self::Error>;

    fn gas(&self) -> i64;
    fn set_gas(&mut self, gas: i64);

    fn registers(&self) -> [u64; NUMBER_OF_REGISTERS];
    fn set_registers(&mut self, registers: &[u64; NUMBER_OF_REGISTERS]);

    fn program_counter(&self) -> Option<u32>;
    fn set_next_program_counter(&mut self, pc: u32);

    fn set_program(&mut self, code: &[u8], container: ProgramContainer) -> Result<(), Self::Error>;

    fn set_page(&mut self, page: u32, access: MemoryAccess);
    fn read_memory(&self, address: u32, out: &mut [u8]) -> Result<(), Self::Error>;
    fn write_memory(&mut self, address: u32, data: &[u8]) -> Result<(), Self::Error>;
}
