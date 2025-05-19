use mago_ast::Expression;
use mago_ast::Hint;
use mago_ast::Identifier;
use mago_span::HasSpan;
use mago_walker::Walker;

use super::context::Context;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct DefinitionFindingWalker;

// use walk_in_* instead of walk_*, because it avoids overloading default trait behavior, wich could stop the walking
impl<'a> Walker<Context<'a>> for DefinitionFindingWalker {
    // TODO probleme : ca prend aussi les variables dans des signatures de fonction...
    fn walk_in_direct_variable(&self, direct_variable: &mago_ast::DirectVariable, context: &mut Context<'a>) {
        if direct_variable.span().has_offset(*context.offset) {
            context.identifiers.push(Identifier::Local(mago_ast::LocalIdentifier {
                span: direct_variable.span(),
                value: direct_variable.name,
            }));
        }
    }

    fn walk_in_function_call(&self, function_call: &mago_ast::FunctionCall, context: &mut Context<'a>) {
        match function_call.function.as_ref() {
            mago_ast::Expression::Identifier(identifier) => {
                if identifier.span().has_offset(*context.offset) {
                    context.identifiers.push(identifier.clone());
                }
            }
            _ => todo!(),
        }
    }

    fn walk_in_method_call(&self, method_call: &mago_ast::MethodCall, context: &mut Context<'a>) {
        match &method_call.method {
            mago_ast::ClassLikeMemberSelector::Identifier(local_identifier) => {
                if local_identifier.span().has_offset(*context.offset) {
                    context.identifiers.push(Identifier::Local(local_identifier.clone()));
                }
            }
            mago_ast::ClassLikeMemberSelector::Variable(_variable) => todo!("variable"),
            mago_ast::ClassLikeMemberSelector::Expression(_class_like_member_expression_selector) => {
                todo!("class_like_member_expression_selector")
            }
        }
    }

    fn walk_in_static_method_call(&self, static_method_call: &mago_ast::StaticMethodCall, context: &mut Context<'a>) {
        match &static_method_call.method {
            mago_ast::ClassLikeMemberSelector::Identifier(local_identifier) => {
                if local_identifier.span().has_offset(*context.offset) {
                    context.identifiers.push(Identifier::Local(local_identifier.clone()));
                }
            }
            mago_ast::ClassLikeMemberSelector::Variable(_variable) => todo!("variable"),
            mago_ast::ClassLikeMemberSelector::Expression(_class_like_member_expression_selector) => {
                todo!("class_like_member_expression_selector")
            }
        }

        if static_method_call.class.span().has_offset(*context.offset) {
            if let Expression::Identifier(i) = static_method_call.class.as_ref() {
                context.identifiers.push(i.clone());
            }
        }
    }

    fn walk_in_instantiation(&self, instantiation: &mago_ast::Instantiation, context: &mut Context<'a>) {
        if instantiation.class.span().has_offset(*context.offset) {
            if let mago_ast::Expression::Identifier(identifier) = &instantiation.class.as_ref() {
                context.identifiers.push(identifier.clone());
            }
        }
    }

    fn walk_in_use_item(&self, use_item: &mago_ast::UseItem, context: &mut Context<'a>) {
        if use_item.name.span().has_offset(*context.offset) {
            context.identifiers.push(use_item.name.clone());
        }
        if let Some(_alias) = &use_item.alias {
            // todo: not link ? -> list des refs dans le fichier
        }
    }

    fn walk_in_implements(&self, implements: &mago_ast::Implements, context: &mut Context<'a>) {
        for r#type in &implements.types.nodes {
            if r#type.span().has_offset(*context.offset) {
                context.identifiers.push(r#type.clone());
            }
        }
    }

    fn walk_in_extends(&self, extends: &mago_ast::Extends, context: &mut Context<'a>) {
        for r#type in &extends.types.nodes {
            if r#type.span().has_offset(*context.offset) {
                context.identifiers.push(r#type.clone());
            }
        }
    }

    fn walk_in_constant_access(&self, constant_access: &mago_ast::ConstantAccess, context: &mut Context<'a>) {
        if constant_access.span().has_offset(*context.offset) {
            context.identifiers.push(constant_access.name.clone());
        }
    }

    fn walk_in_hint(&self, hint: &mago_ast::Hint, context: &mut Context<'a>) {
        if let Hint::Identifier(identifier) = hint {
            if identifier.span().has_offset(*context.offset) {
                context.identifiers.push(identifier.clone());
            }
        }
    }
}
