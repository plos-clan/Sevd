mod define;
mod lets;

use lets::resolution_global;

use crate::compiler::com_error::SematicError;
use crate::compiler::ir::hast::{HastFunction, HastItem};
use crate::compiler::ir::{AnnotationElement, AstNode, TypeRef};
use crate::compiler::lexer::Token;
use crate::compiler::sematic::Semantic;
use crate::compiler::sematic::resolution::define::{collect_enum, collect_struct};
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

fn generic_resolution(
    semantic: &mut Semantic,
    func: AstNode,
    attributes: Vec<(Token, Vec<AnnotationElement>)>,
) -> HastItem {
    let AstNode::Function {
        name,
        generics,
        constraint,
        ret_type,
        args,
        block,
    } = func
    else {
        unreachable!()
    };

    let constraint_opt = if let Some(generics) = generics {
        let mut constraints_target = vec![];
        for token in generics {
            let token_text = token.get_span().text();
            let constraints = constraint.as_deref().unwrap_or(&[]);
            if let Some((_, my_type)) = constraints
                .iter()
                .find(|(token, _)| token.get_span().text() == token_text)
            {
                constraints_target.push((token, my_type.clone()));
            } else {
                constraints_target.push((token, vec![TypeRef::Understood]));
            }
        }
        Some(constraints_target)
    } else {
        if constraint.is_some() {
            semantic.push_error(SematicError::MissingGenericConstraint(name.clone()));
        }
        None
    };

    HastItem::Function(Box::new(HastFunction {
        name,
        args: argument_to_iterm(args),
        ret: ret_type,
        attributes,
        constraint: constraint_opt,
        body: block.as_ref().clone(),
    }))
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
        } => Some(collect_struct(semantic, name, generics, fields, attributes)),
        AstNode::EnumDefine { name, variants } => {
            Some(collect_enum(semantic, name, variants, attributes))
        }
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
