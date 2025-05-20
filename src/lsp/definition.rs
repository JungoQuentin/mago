use mago_syntax::{
    ast::{Identifier, Program},
    walker::Walker,
};

use context::Context;
use walker::DefinitionFindingWalker;

mod context;
mod walker;

#[derive(Debug, Clone)]
pub struct DefinitionFinder;

impl DefinitionFinder {
    pub fn find(&self, program: &Program, offset: usize) -> Vec<Identifier> {
        let mut context = Context::new(&offset);
        DefinitionFindingWalker.walk_program(program, &mut context);
        context.take_identifiers()
    }
}
