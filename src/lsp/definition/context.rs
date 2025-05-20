use mago_syntax::ast::Identifier;

#[derive(Debug, Clone)]
pub struct Context<'a> {
    pub offset: &'a usize,
    pub identifiers: Vec<Identifier>,
}

impl<'a> Context<'a> {
    pub fn new(offset: &'a usize) -> Self {
        Self { offset, identifiers: Vec::new() }
    }

    pub fn take_identifiers(&mut self) -> Vec<Identifier> {
        std::mem::take(&mut self.identifiers)
    }
}
