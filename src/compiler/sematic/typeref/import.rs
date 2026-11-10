use crate::compiler::com_error::SematicError;
use crate::compiler::ir::hast::HastItem;
use crate::compiler::ir::refast::{ImportedName, MemberDefId, TypeDefId};
use crate::compiler::lexer::Token;
use crate::compiler::module::ModuleId;
use crate::compiler::sematic::Semantic;

fn collect_import_unit(
    semantic: &mut Semantic,
    names: Vec<Token>,
    module_id: ModuleId,
) -> Vec<ImportedName> {
    let mut imports = vec![];
    let mut errors = vec![];

    for name in names {
        let Some(module) = semantic.symtbl.compiler_mut().module_mut(module_id) else {
            unreachable!()
        };
        let member_id = module
            .members
            .iter()
            .find(|(_, token)| token.get_span().text() == name.get_span().text());
        let item_id = module
            .items
            .iter()
            .find(|(_, token)| token.name.get_span().text() == name.get_span().text());

        match (member_id, item_id) {
            (Some(member), Some(item)) => imports.push(ImportedName::Double(
                name,
                TypeDefId {
                    module: module_id,
                    item: item.0,
                },
                MemberDefId {
                    module: module_id,
                    member: member.0,
                },
            )),
            (Some(member), None) => imports.push(ImportedName::MemberId(
                name,
                MemberDefId {
                    module: module_id,
                    member: member.0,
                },
            )),
            (None, Some(item)) => imports.push(ImportedName::TypeId(
                name,
                TypeDefId {
                    module: module_id,
                    item: item.0,
                },
            )),
            (None, None) => {
                errors.push(SematicError::UnresolvedSymbol(name));
            }
        }
    }

    semantic.errors.append(&mut errors);
    imports
}

pub fn collect_import(semantic: &mut Semantic, items: &[HastItem]) -> Vec<ImportedName> {
    let mut imports = vec![];
    for item in items {
        if let HastItem::Import { name, module_id } = item {
            imports.append(&mut collect_import_unit(semantic, name.clone(), *module_id));
        }
    }
    imports
}
