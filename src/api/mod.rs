
mod common;
pub mod collection;
pub mod polkavm;
pub mod stdin;

pub const NUMBER_OF_REGISTERS: usize = 13;

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum Status {
    Ok = 255,
    Halt = 0,
    Trap = 1,
    Fault = 2,
    Host = 3,
    OutOfGas = 4,
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Status::Ok => "ok",
            Status::Halt => "halt",
            Status::Trap => "trap",
            Status::Fault => "fault",
            Status::Host => "host",
            Status::OutOfGas => "out-of-gas",
        })
    }
}

#[derive(Copy, Clone, Debug)]
pub enum ProgramContainer {
    Generic,
    #[allow(dead_code)]
    SPI,
    #[allow(dead_code)]
    PolkaVM,
}

#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
pub enum MemoryAccess {
    Readable,
    Writeable,
}

#[derive(Debug)]
pub enum Error {
    InvalidProgram,
    UnsupportedContainer,
    Other(String),
    Wrap(Box<dyn std::error::Error + Sync + Send>),
}

impl Error {
    pub fn wrap<E: 'static + std::error::Error + Sync + Send>(other: E) -> Self {
        Self::Wrap(Box::new(other))
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidProgram => write!(f, "invalid program"),
            Error::UnsupportedContainer => write!(f, "unsupported container"),
            Error::Other(s) => write!(f, "Other: {s}"),
            Error::Wrap(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;


/// Low-level PVM interface.
pub trait PvmApi {
    fn run(&mut self) -> Result<Status>;

    #[allow(dead_code)]
    // TODO [ToDr] Just to show what the API could look like for the PVM Debugger.
    fn step(&mut self) -> Result<Status> {
        self.run()
    }

    fn gas(&self) -> i64;
    fn set_gas(&mut self, gas: i64);

    fn registers(&self) -> [u64; NUMBER_OF_REGISTERS];
    fn set_registers(&mut self, registers: &[u64; NUMBER_OF_REGISTERS]);

    fn program_counter(&self) -> Option<u32>;
    fn set_next_program_counter(&mut self, pc: u32);

    fn set_program(&mut self, code: &[u8], container: ProgramContainer) -> Result<()>;

    fn set_page(&mut self, page: u32, access: MemoryAccess);
    #[allow(dead_code)]
    fn read_memory(&self, address: u32, out: &mut [u8]) -> Result<()>;
    fn write_memory(&mut self, address: u32, data: &[u8]) -> Result<()>;
}
