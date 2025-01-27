use super::PvmApi;

pub struct PvmApiCollection {
    collection: Vec<Box<dyn PvmApi>>,
}

impl PvmApiCollection {
    pub fn new(collection: Vec<Box<dyn PvmApi>>) -> Self {
        assert!(!collection.is_empty());

        Self { collection }
    }

    fn for_all_mut<F, G, R>(&mut self, mut run: F, merge: G) -> R
    where
        F: FnMut(&mut dyn PvmApi) -> R,
        G: Fn(R, R) -> R,
    {
        let mut result = run(self.collection[0].as_mut());
        for pvm in &mut self.collection[1..] {
            result = merge(result, run(pvm.as_mut()));
        }
        result
    }

    fn for_all<F, G, R>(&self, run: F, merge: G) -> R
    where
        F: Fn(&dyn PvmApi) -> R,
        G: Fn(R, R) -> R,
    {
        let mut result = run(self.collection[0].as_ref());
        for pvm in &self.collection[1..] {
            result = merge(result, run(pvm.as_ref()));
        }
        result
    }
}

fn propagate<R: core::fmt::Debug + Eq>(a: R, b: R, ctx: &str) -> R {
    if a != b {
        log::error!("[{ctx}] PVM status mismatch: {a:?} vs {b:?}");
    }
    a
}
fn propagate_res<R: core::fmt::Debug + Eq>(a: super::Result<R>, b: super::Result<R>, ctx: &str) -> super::Result<R> {
    match (a, b) {
        (Err(e), o) | (o, Err(e)) => {
            log::error!("[{ctx}] Ignoring result due to error in one of the PVMs: {o:?}");
            Err(e)
        }
        (Ok(a), Ok(b)) => Ok(propagate(a, b, ctx)),
    }
}

impl PvmApi for PvmApiCollection {
    fn run(&mut self) -> super::Result<super::Status> {
        self.for_all_mut(|p| p.run(), |a, b| propagate_res(a, b, "run"))
    }

    fn gas(&self) -> i64 {
        self.for_all(|p| p.gas(), |a, b| propagate(a, b, "gas"))
    }

    fn set_gas(&mut self, gas: i64) {
        self.for_all_mut(|p| p.set_gas(gas), |a, b| propagate(a, b, "set_gas"))
    }

    fn registers(&self) -> [u64; super::NUMBER_OF_REGISTERS] {
        self.for_all(|p| p.registers(), |a, b| propagate(a, b, "registers"))
    }

    fn set_registers(&mut self, registers: &[u64; super::NUMBER_OF_REGISTERS]) {
        self.for_all_mut(|p| p.set_registers(registers), |a, b| propagate(a, b, "set_registers"))
    }

    fn program_counter(&self) -> Option<u32> {
        self.for_all(|p| p.program_counter(), |a, b| propagate(a, b, "PC"))
    }

    fn set_next_program_counter(&mut self, pc: u32) {
        self.for_all_mut(|p| p.set_next_program_counter(pc), |a, b| propagate(a, b, "set_PC"))
    }

    fn set_program(&mut self, code: &[u8], container: super::ProgramContainer) -> super::Result<()> {
        self.for_all_mut(
            |p| p.set_program(code, container),
            |a, b| propagate_res(a, b, "set_program"),
        )
    }

    fn set_page(&mut self, page: u32, access: super::MemoryAccess) {
        self.for_all_mut(|p| p.set_page(page, access), |a, b| propagate(a, b, "set_page"))
    }

    fn read_memory(&self, _address: u32, _out: &mut [u8]) -> super::Result<()> {
        todo!()
    }

    fn write_memory(&mut self, address: u32, data: &[u8]) -> super::Result<()> {
        self.for_all_mut(
            |p| p.write_memory(address, data),
            |a, b| propagate_res(a, b, "write_memory"),
        )
    }
}
