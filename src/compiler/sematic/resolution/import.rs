use crate::compiler::com_error::SematicError;
use crate::compiler::ir::hast::HastItem;
use crate::compiler::lexer::{Token, TokenType};
use crate::compiler::module::ModuleId;
use crate::compiler::sematic::Semantic;
use std::path::Path;

fn import_module_path(path: &str) -> Option<Vec<String>> {
    let parts = if path.contains('/') {
        if Path::new(path).is_absolute() {
            return None;
        }
        let mut parts: Vec<String> = path.split('/').map(str::to_owned).collect();
        let filename = parts.last()?.clone();
        if filename == "mod.sev" {
            parts.pop();
        } else if let Some(name) = filename.strip_suffix(".sev") {
            name.clone_into(parts.last_mut()?);
        }
        parts
    } else {
        path.split('.').map(str::to_owned).collect()
    };
    if parts.is_empty()
        || parts
            .iter()
            .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return None;
    }
    Some(parts)
}

fn parser_path(semantic: &mut Semantic, path_token: Token) -> Option<ModuleId> {
    let ctoken = path_token.clone();
    let TokenType::String(path) = ctoken.get_type() else {
        semantic.push_error(SematicError::IllegalImport(path_token));
        return None;
    };
    let Some(parts) = import_module_path(path.as_str()) else {
        semantic.push_error(SematicError::UnknownLibrary(path_token, path.clone()));
        return None;
    };
    let Some(module) = semantic.symtbl.compiler().resolve_module(&parts) else {
        semantic.push_error(SematicError::UnknownLibrary(path_token, path.clone()));
        return None;
    };
    if module.file.is_none() {
        semantic.push_error(SematicError::UnknownLibrary(path_token, path.clone()));
        return None;
    }
    Some(module.id)
}

pub fn resolution_import(semantic: &mut Semantic, name: Vec<Token>, path: Token) -> HastItem {
    let Some(id) = parser_path(semantic, path) else {
        return HastItem::Error;
    };
    HastItem::Import {
        name,
        module_id: id,
    }
}
