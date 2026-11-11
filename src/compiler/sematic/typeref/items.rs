use crate::compiler::com_error::SematicError;
use crate::compiler::ir::TypeRef;
use crate::compiler::ir::hast::{HastEnum, HastStruct};
use crate::compiler::ir::hir::{FieldId, ItemId, TypeVarId};
use crate::compiler::ir::refast::{
    ImportedName, RefAst, RefAstType, RefEnum, RefStruct, TypeDefId,
};
use crate::compiler::ir::tnode::PrimitiveType;
use crate::compiler::lexer::Token;
use crate::compiler::sematic::Semantic;

fn is_generics(source: &[(Token, TypeVarId)], target: &Token) -> Option<TypeVarId> {
    source
        .iter()
        .find(|(name, _)| name.get_span().text() == target.get_span().text())
        .map(|(_, id)| *id)
}

fn lookup_type_def(name: &Token, imports: &[ImportedName]) -> Result<TypeDefId, SematicError> {
    let mut error = None;
    let id = imports.iter().find_map(|import| {
        if import.get_token().get_span().text() != name.get_span().text() {
            return None;
        }
        match import {
            ImportedName::TypeId(_, id) => Some(*id),
            ImportedName::MemberId(_, _) => {
                error = Some(SematicError::MemberNoAppliedType(name.clone()));
                None
            }
            ImportedName::Double(_, tid, _) => Some(*tid),
        }
    });

    if let Some(error) = error {
        return Err(error);
    }
    id.ok_or(SematicError::UnresolvedSymbol(name.clone()))
}

fn ref_cast_unit(
    semantic: &mut Semantic,
    raw_type: TypeRef,
    imports: &[ImportedName],
    generics: &[(Token, TypeVarId)],
) -> RefAstType {
    match raw_type {
        TypeRef::Ident(name) => {
            if let Some(var) = is_generics(generics, &name) {
                // 泛型检查
                return RefAstType::Infer(var);
            }
            if let Some(p_type) = PrimitiveType::cast(name.get_span().text()) {
                // 基元检查
                return RefAstType::Primitive(p_type);
            }

            match lookup_type_def(&name, imports) {
                Ok(def) => RefAstType::Single(def),
                Err(error) => {
                    semantic.push_error(error);
                    RefAstType::Primitive(PrimitiveType::Error)
                }
            }
        }
        TypeRef::Tuple(types) => RefAstType::Tuple(
            types
                .into_iter()
                .map(|raw_type| ref_cast_unit(semantic, raw_type, imports, generics))
                .collect(),
        ),
        TypeRef::Understood => {
            RefAstType::Understood(semantic.symtbl.understood_mut().alloc_var_id())
        }
        TypeRef::Named {
            name,
            generics: g_named,
        } => {
            if is_generics(generics, &name).is_some()
                || PrimitiveType::cast(name.get_span().text()).is_some()
            // 泛型回填与基元不支持泛型参数
            {
                semantic.push_error(SematicError::InvalidGeneratorPassing(name));
                return RefAstType::Primitive(PrimitiveType::Error);
            }

            match lookup_type_def(&name, imports) {
                Ok(def) => {
                    let args = g_named
                        .into_iter()
                        .map(|arg| ref_cast_unit(semantic, arg, imports, generics))
                        .collect();
                    RefAstType::Adt { def, args }
                }
                Err(error) => {
                    semantic.push_error(error);
                    RefAstType::Primitive(PrimitiveType::Error)
                }
            }
        }
    }
}

pub fn ref_struct(
    semantic: &mut Semantic,
    id: ItemId,
    stc: HastStruct,
    imports: &[ImportedName],
) -> RefAst {
    let (raw_id, types): (Vec<FieldId>, Vec<TypeRef>) = stc.fields.into_iter().unzip();

    let generic_names = stc.generics.unwrap_or_default();
    let generic: Vec<(Token, TypeVarId)> = generic_names
        .into_iter()
        .map(|name| (name, semantic.symtbl.infer_mut().alloc_var_id()))
        .collect();

    let fields_types: Vec<RefAstType> = types
        .into_iter()
        .map(|raw_type| ref_cast_unit(semantic, raw_type, imports, &generic))
        .collect();
    RefAst::Struct(
        id,
        RefStruct {
            fields: raw_id.into_iter().zip(fields_types).collect(),
        },
    )
}

pub fn ref_enum(
    semantic: &mut Semantic,
    id: ItemId,
    enm: HastEnum,
    imports: &[ImportedName],
) -> RefAst {
    let (raw_id, types): (Vec<FieldId>, Vec<Vec<TypeRef>>) = enm.variants.into_iter().unzip();
    let generic_names = enm.generics.unwrap_or_default();
    let generic: Vec<(Token, TypeVarId)> = generic_names
        .into_iter()
        .map(|name| (name, semantic.symtbl.infer_mut().alloc_var_id()))
        .collect();

    let var_types: Vec<Vec<RefAstType>> = types
        .into_iter()
        .map(|raw_types| {
            raw_types
                .into_iter()
                .map(|raw_type| ref_cast_unit(semantic, raw_type, imports, &generic))
                .collect()
        })
        .collect();

    RefAst::Enum(
        id,
        RefEnum {
            variants: raw_id.into_iter().zip(var_types).collect(),
        },
    )
}
