use crate::compiler::ir::hast::{HastEnum, HastItem, HastStruct};
use crate::compiler::ir::hir::{ERROR_FIELD_ID, FieldId, ItemId};
use crate::compiler::ir::{AnnotationElement, TypeRef};
use crate::compiler::lexer::Token;
use crate::compiler::sematic::Semantic;
use crate::compiler::symtbl::ItemType;

fn add_item(semantic: &mut Semantic, name: Token, types: ItemType) -> ItemId {
    let module_id = semantic.symtbl.current_module();
    let Some(module) = semantic.symtbl.compiler_mut().module_mut(module_id) else {
        unreachable!()
    };
    let id = module.alloc_item_id();
    module
        .collect_item_define(id, name, types)
        .unwrap_or_else(|err| semantic.push_error(err));
    id
}

fn collect_field(semantic: &mut Semantic, id: ItemId, fields: Vec<Token>) -> Vec<FieldId> {
    let mut fields_id = vec![];

    for field in fields {
        let id = {
            let module_id = semantic.symtbl.current_module();
            let Some(module) = semantic.symtbl.compiler_mut().module_mut(module_id) else {
                unreachable!()
            };
            module.collect_item_field(id, field)
        }
        .unwrap_or_else(|err| {
            semantic.push_error(err);
            ERROR_FIELD_ID
        });
        fields_id.push(id);
    }

    fields_id
}

pub fn collect_struct(
    semantic: &mut Semantic,
    name: Token,
    generics: Option<Vec<Token>>,
    fields: &[(Token, TypeRef)],
    attributes: Vec<(Token, Vec<AnnotationElement>)>,
) -> HastItem {
    let id = add_item(semantic, name, ItemType::Struct);
    let members = fields.iter().map(|(t, _)| t.clone()).collect();
    let refs: Vec<TypeRef> = fields
        .iter()
        .map(|(_, type_ref)| type_ref.clone())
        .collect();
    let ids = collect_field(semantic, id, members);
    HastItem::Struct(
        id,
        HastStruct {
            generics,
            fields: ids.into_iter().zip(refs).collect(),
            attributes,
        },
    )
}

pub fn collect_enum(
    semantic: &mut Semantic,
    name: Token,
    variants: &[(Token, Vec<TypeRef>)],
    attributes: Vec<(Token, Vec<AnnotationElement>)>,
) -> HastItem {
    let id = add_item(semantic, name, ItemType::Enum);
    let fields = variants.iter().map(|(t, _)| t.clone()).collect();
    let refs: Vec<Vec<TypeRef>> = variants
        .iter()
        .map(|(_, type_ref)| type_ref.clone())
        .collect();
    let ids = collect_field(semantic, id, fields);
    HastItem::Enum(
        id,
        HastEnum {
            variants: ids.into_iter().zip(refs).collect(),
            attributes,
        },
    )
}
