use crate::compiler::com_error::SematicError;
use crate::compiler::ir::hast::{HastEnum, HastFunction, HastGlobal, HastItem, HastStruct};
use crate::compiler::ir::{AnnotationElement, AstNode, GenericArg};
use crate::compiler::lexer::Token;
use crate::compiler::sematic::Semantic;
use std::iter::Peekable;
use std::vec::IntoIter;

fn argument_to_iterm(args: Vec<AstNode>) -> Vec<(Token, GenericArg)> {
    let mut arguments = vec![];
    for node in args {
        let AstNode::Define { name, type_name } = node else {
            unreachable!()
        };
        let ty = if let Some(name) = type_name {
            name
        } else {
            GenericArg::Understood
        };
        arguments.push((name, ty));
    }
    arguments
}

fn generic_resolution(
    func: AstNode,
    attributes: Vec<(Token, Vec<AnnotationElement>)>,
) -> Result<HastItem, SematicError> {
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
                constraints_target.push((token, vec![GenericArg::Understood]));
            }
        }
        Some(constraints_target)
    } else {
        if constraint.is_some() {
            return Err(SematicError::MissingGenericConstraint(name));
        }
        None
    };

    Ok(HastItem::Function(Box::new(HastFunction {
        name,
        args: argument_to_iterm(args),
        ret: ret_type,
        attributes,
        constraint: constraint_opt,
        body: block.as_ref().clone(),
    })))
}

fn semantic_item(iter: &mut Peekable<IntoIter<AstNode>>) -> Result<Option<HastItem>, SematicError> {
    let mut attributes = vec![];

    loop {
        let Some(node) = iter.peek() else {
            if attributes.is_empty() {
                return Ok(None);
            };
            let node: &(Token, Vec<AnnotationElement>) = attributes.last().unwrap();
            return Err(SematicError::InvalidAnnotationTarget(node.0.clone()));
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
            return Ok(None);
        };
        let (token, _) = attributes.last().unwrap();
        return Err(SematicError::InvalidAnnotationTarget(token.clone()));
    };
    match node {
        AstNode::Function { .. } => Ok(Some(generic_resolution(node, attributes)?)),
        AstNode::StructDefine {
            name,
            generics,
            fields,
        } => Ok(Some(HastItem::Struct(HastStruct {
            name,
            generics,
            fields,
            attributes,
        }))),
        AstNode::EnumDefine { name, variants } => Ok(Some(HastItem::Enum(HastEnum {
            name,
            attributes,
            variants,
        }))),
        AstNode::DefineElse {
            token,
            head,
            type_name,
            vars,
            el_blk,
        } => {
            let Some(vars) = vars else {
                return Err(SematicError::MissingInitializer(token));
            };
            let el = el_blk.map(|item| item.as_ref().clone());
            Ok(Some(HastItem::Global(Box::new(HastGlobal {
                pat: head,
                ty: type_name.unwrap_or(GenericArg::Understood),
                attributes,
                init: vars.as_ref().clone(),
                el,
            }))))
        }
        _ => unreachable!(),
    }
}

pub fn resolution(semantic: &mut Semantic, nodes: Vec<AstNode>) -> Vec<HastItem> {
    let mut iter = nodes.into_iter().peekable();
    let mut items = vec![];
    loop {
        match semantic_item(&mut iter) {
            Ok(Some(hast_item)) => items.push(hast_item),
            Err(error) => semantic.push_error(error),
            _ => break,
        }
    }
    items
}
