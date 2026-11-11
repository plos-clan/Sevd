use crate::compiler::com_error::SematicError;
use crate::compiler::ir::hast::HastItem;
use crate::compiler::ir::refast::{ImportedName, MemberDefId, TypeDefId};
use crate::compiler::lexer::Token;
use crate::compiler::module::ModuleId;
use crate::compiler::sematic::Semantic;
use std::collections::HashMap;

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

/// 导入当前源文件已经定义的成员和类型
fn collect_current_import(semantic: &Semantic) -> Vec<ImportedName> {
    let current_id = semantic.symtbl.current_module();
    let current_mod = semantic
        .symtbl
        .compiler()
        .module(current_id)
        .expect("current module must exist");

    let mut members: HashMap<_, _> = current_mod
        .members
        .iter()
        .map(|(id, token)| (token.get_span().text(), *id))
        .collect();

    let mut names = Vec::new();

    for (id, item) in &current_mod.items {
        let type_id = TypeDefId {
            module: current_id,
            item: *id,
        };

        let binding = match members.remove(item.name.get_span().text()) {
            Some(member) => ImportedName::Double(
                item.name.clone(),
                type_id,
                MemberDefId {
                    module: current_id,
                    member,
                },
            ),
            None => ImportedName::TypeId(item.name.clone(), type_id),
        };

        names.push(binding);
    }

    for (id, token) in &current_mod.members {
        if members.remove(token.get_span().text()).is_some() {
            names.push(ImportedName::MemberId(
                token.clone(),
                MemberDefId {
                    module: current_id,
                    member: *id,
                },
            ));
        }
    }
    names
}

pub fn collect_import(semantic: &mut Semantic, items: &[HastItem]) -> Vec<ImportedName> {
    let mut imports = vec![];
    for item in items {
        if let HastItem::Import { name, module_id } = item {
            imports.append(&mut collect_import_unit(semantic, name.clone(), *module_id));
        }
    }
    imports.append(&mut collect_current_import(semantic));
    imports
}
