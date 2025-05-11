use mago_ast::Identifier;
use mago_ast::Program;

#[derive(Debug, Clone)]
pub struct Context<'a> {
    pub offset: &'a usize,
    pub program: &'a Program,
    pub identifiers: Vec<Identifier>,
}

impl<'a> Context<'a> {
    pub fn new(offset: &'a usize, program: &'a Program) -> Self {
        Self { offset, program, identifiers: Vec::new() }
    }

    pub fn take_identifiers(&mut self) -> Vec<Identifier> {
        std::mem::take(&mut self.identifiers)
    }
}
