use crate::compiler::com_error::SematicError;
use crate::compiler::ir::hast::{HastFunction, HastGlobal, HastItem, HastPattern};
use crate::compiler::ir::hir::MemberId;
use crate::compiler::ir::{AnnotationElement, AstNode, ExprNode, Pattern, TypeRef};
use crate::compiler::lexer::Token;
use crate::compiler::sematic::Semantic;
use crate::compiler::sematic::resolution::argument_to_iterm;

fn add_member(semantic: &mut Semantic, name: Token) -> MemberId {
    let module_id = semantic.symtbl.current_module();
    let Some(module) = semantic.symtbl.compiler_mut().module_mut(module_id) else {
        unreachable!()
    };
    let id = module.alloc_member_id();
    module
        .collect_member_define(id, name)
        .unwrap_or_else(|err| semantic.push_error(err));
    id
}

fn semantic_global_pattern(semantic: &mut Semantic, head: Pattern) -> HastPattern {
    match head {
        Pattern::Bind(name) => HastPattern::GlobalBind(add_member(semantic, name.clone())),
        Pattern::Tuple(tuple) => {
            let hast_tuple = tuple
                .into_iter()
                .map(|item| semantic_global_pattern(semantic, item))
                .collect();
            HastPattern::Tuple(hast_tuple)
        }
        head => {
            semantic.push_error(SematicError::InvalidBinding(
                head.first_token().unwrap().clone(),
            ));
            HastPattern::Error
        }
    }
}

pub fn resolution_global(
    semantic: &mut Semantic,
    node: AstNode,
    attributes: Vec<(Token, Vec<AnnotationElement>)>,
) -> HastItem {
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
    let vars = vars.unwrap_or_else(|| {
        semantic.push_error(SematicError::MissingInitializer(token.clone()));
        Box::new(ExprNode::Error)
    });
    let el = el_blk.map(|item| item.as_ref().clone());
    let pattern = semantic_global_pattern(semantic, head);

    HastItem::Global(Box::new(HastGlobal {
        pat: pattern,
        ty: type_name.unwrap_or(TypeRef::Understood),
        attributes,
        init: vars.as_ref().clone(),
        el,
    }))
}

pub fn generic_resolution(
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

    let id = add_member(semantic, name);

    HastItem::Function(
        id,
        Box::new(HastFunction {
            args: argument_to_iterm(args),
            ret: ret_type,
            attributes,
            constraint: constraint_opt,
            body: block.as_ref().clone(),
        }),
    )
}
