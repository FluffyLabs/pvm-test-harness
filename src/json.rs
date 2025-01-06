#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Page {
    pub address: u32,
    pub length: u32,
    pub is_writable: bool,
}

#[derive(PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct MemoryChunk {
    pub address: u32,
    pub contents: Vec<u8>,
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct TestcaseJson {
    pub name: String,
    pub initial_regs: [u64; 13],
    pub initial_pc: u32,
    pub initial_page_map: Vec<Page>,
    pub initial_memory: Vec<MemoryChunk>,
    pub initial_gas: i64,
    pub program: Vec<u8>,
    pub expected_status: String,
    pub expected_regs: Vec<u64>,
    pub expected_pc: u32,
    pub expected_memory: Vec<MemoryChunk>,
    pub expected_gas: i64,
}
