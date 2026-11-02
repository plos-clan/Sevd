use crate::compiler::com_error::SematicError;
use crate::compiler::ir::hast::{HastGlobal, HastItem};
use crate::compiler::ir::{AnnotationElement, AstNode, TypeRef};
use crate::compiler::lexer::Token;
use crate::compiler::sematic::Semantic;

pub fn resolution_global(
    semantic: &mut Semantic,
    node: AstNode,
    attributes: Vec<(Token, Vec<AnnotationElement>)>,
) -> Result<HastItem, SematicError> {
    let AstNode::DefineElse {
        token,
        head,
        type_name,
        vars,
        el_blk,
    } = node
    else {
        unreachable!()
    };
    let Some(vars) = vars else {
        return Err(SematicError::MissingInitializer(token));
    };
    let el = el_blk.map(|item| item.as_ref().clone());

    Ok(HastItem::Global(Box::new(HastGlobal {
        pat: head,
        ty: type_name.unwrap_or(TypeRef::Understood),
        attributes,
        init: vars.as_ref().clone(),
        el,
    })))
}
