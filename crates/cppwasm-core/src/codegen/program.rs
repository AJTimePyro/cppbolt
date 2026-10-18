use super::*;

pub(super) struct Plan<'p> {
    pub(super) candidates: Vec<FuncCandidate<'p>>,
    pub(super) reachable: HashSet<usize>,
    pub(super) address_taken: HashSet<String>,
    pub(super) need_printf: bool,
    pub(super) need_alloc: bool,
    pub(super) need_table: bool,
    pub(super) trap_func_idx: Option<u32>,
}

impl<'a> Codegen<'a> {
    pub fn compile_program(mut self, program: &Program) -> Result<Vec<u8>, CompileError> {
        let plan = self.prepare(program);
        self.emit(&plan)?;
        Ok(self.module.encode())
    }
}
