use super::ProgramContainer;

#[derive(Debug, Default)]
pub(crate) struct InitialState {
    pub registers: [u64; super::NUMBER_OF_REGISTERS],
    pub gas: i64,
    pub pc: u32,
    pub program: Vec<u8>,
    pub container: Option<ProgramContainer>,
}

#[derive(Debug, Default)]
pub(crate) struct OutputState {
    pub registers: [u64; super::NUMBER_OF_REGISTERS],
    pub gas: i64,
    pub pc: Option<u32>,
}
