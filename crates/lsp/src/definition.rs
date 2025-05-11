use context::Context;
use mago_ast::Identifier;
use mago_ast::Program;

use mago_walker::Walker;
use walker::DefinitionFindingWalker;

mod context;
mod walker;

#[derive(Debug, Clone)]
pub struct DefinitionFinder;

impl DefinitionFinder {
    pub fn find(&self, program: &Program, offset: usize) -> Vec<Identifier> {
        let mut context = Context::new(&offset, program);
        DefinitionFindingWalker.walk_program(&program, &mut context);
        context.take_identifiers()
    }
}
