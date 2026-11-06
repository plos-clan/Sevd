mod define;
mod member;

use member::resolution_global;

use crate::compiler::com_error::SematicError;
use crate::compiler::ir::hast::HastItem;
use crate::compiler::ir::{AnnotationElement, AstNode, TypeRef};
use crate::compiler::lexer::Token;
use crate::compiler::sematic::Semantic;
use crate::compiler::sematic::resolution::define::{collect_enum, collect_struct};
use crate::compiler::sematic::resolution::member::generic_resolution;
use std::iter::Peekable;
use std::vec::IntoIter;

fn argument_to_iterm(args: Vec<AstNode>) -> Vec<(Token, TypeRef)> {
    let mut arguments = vec![];
    for node in args {
        let AstNode::Define { name, type_name } = node else {
            unreachable!()
        };
        let ty = if let Some(name) = type_name {
            name
        } else {
            TypeRef::Understood
        };
        arguments.push((name, ty));
    }
    arguments
}

fn semantic_item(
    semantic: &mut Semantic,
    iter: &mut Peekable<IntoIter<AstNode>>,
) -> Option<HastItem> {
    let mut attributes = vec![];

    loop {
        let Some(node) = iter.peek() else {
            if attributes.is_empty() {
                return None;
            }
            let node: &(Token, Vec<AnnotationElement>) = attributes.last().unwrap();
            semantic.push_error(SematicError::InvalidAnnotationTarget(node.0.clone()));
            return None;
        };
        let AstNode::Annotation { .. } = node else {
            break;
        };
        let AstNode::Annotation { name, elements } = iter.next().unwrap() else {
            unreachable!()
        };
        attributes.push((name, elements));
    }

    let Some(node) = iter.next() else {
        if attributes.is_empty() {
            return None;
        }
        let (token, _) = attributes.last().unwrap();
        semantic.push_error(SematicError::InvalidAnnotationTarget(token.clone()));
        return None;
    };
    match node {
        AstNode::Import { name, module_id } => {
            if let Some((token, _)) = attributes.first() {
                semantic.push_error(SematicError::InvalidAnnotationTarget(token.clone()));
            }
            Some(HastItem::Import { name, module_id })
        }
        AstNode::Function { .. } => Some(generic_resolution(semantic, node, attributes)),
        AstNode::StructDefine {
            name,
            generics,
            fields,
        } => Some(collect_struct(
            semantic,
            name,
            generics,
            fields.as_slice(),
            attributes,
        )),
        AstNode::EnumDefine { name, variants } => Some(collect_enum(
            semantic,
            name,
            variants.as_slice(),
            attributes,
        )),
        AstNode::DefineElse { .. } => Some(resolution_global(semantic, node, attributes)),
        _ => unreachable!(),
    }
}

pub fn resolution(semantic: &mut Semantic, nodes: Vec<AstNode>) -> Vec<HastItem> {
    let mut iter = nodes.into_iter().peekable();
    let mut items = vec![];
    while let Some(item) = semantic_item(semantic, &mut iter) {
        items.push(item);
    }
    items
}
