use crate::compiler::ir::hast::{HastEnum, HastItem, HastStruct};
use crate::compiler::ir::hir::ItemId;
use crate::compiler::ir::{AnnotationElement, TypeRef};
use crate::compiler::lexer::Token;
use crate::compiler::sematic::Semantic;

fn add_item(semantic: &mut Semantic, name: Token) -> ItemId {
    let module_id = semantic.symtbl.current_module();
    let Some(module) = semantic.symtbl.compiler_mut().module_mut(module_id) else {
        unreachable!()
    };
    let id = module.alloc_item_id();
    module
        .collect_item_define(id, name)
        .unwrap_or_else(|err| semantic.push_error(err));
    id
}

pub fn collect_struct(
    semantic: &mut Semantic,
    name: Token,
    generics: Option<Vec<Token>>,
    fields: Vec<(Token, TypeRef)>,
    attributes: Vec<(Token, Vec<AnnotationElement>)>,
) -> HastItem {
    let id = add_item(semantic, name);
    HastItem::Struct(
        id,
        HastStruct {
            generics,
            fields,
            attributes,
        },
    )
}

pub fn collect_enum(
    semantic: &mut Semantic,
    name: Token,
    variants: Vec<(Token, Vec<TypeRef>)>,
    attributes: Vec<(Token, Vec<AnnotationElement>)>,
) -> HastItem {
    let id = add_item(semantic, name);
    HastItem::Enum(id,HastEnum {
        variants,
        attributes,
    })
}
