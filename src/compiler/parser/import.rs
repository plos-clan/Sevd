use std::path::Path;

use crate::compiler::Compiler;
use crate::compiler::SourceFile;
use crate::compiler::com_error::ParserError;
use crate::compiler::ir::AstNode;
use crate::compiler::lexer::Token;
use crate::compiler::lexer::TokenType;
use crate::compiler::module::SubModule;
use crate::compiler::symtbl::SymbolTable;

use super::Parser;

fn import_error(token: &Token, message: impl Into<String>) -> ParserError {
    ParserError::UnknownLibrary(token.clone(), message.into())
}

fn source_file_name(file: &SourceFile) -> &str {
    Path::new(file.name.as_str())
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(file.name.as_str())
}

fn find_in_submodules<'a>(
    mods: &'a [SubModule],
    parts: &[String],
    path_token: &Token,
) -> Result<&'a SourceFile, ParserError> {
    let Some((current, rest)) = parts.split_first() else {
        return Err(import_error(path_token, "import path is empty"));
    };

    for submodule in mods {
        match submodule {
            SubModule::File(file) => {
                if rest.is_empty() && source_file_name(file) == current {
                    return Ok(file);
                }
            }
            SubModule::SubModule { name, root, mods } if name == current => {
                if rest.is_empty() || (rest.len() == 1 && rest[0] == "mod.sev") {
                    return root.as_ref().ok_or_else(|| {
                        import_error(
                            path_token,
                            format!("submodule `{name}` does not have a root file"),
                        )
                    });
                }
                return find_in_submodules(mods, rest, path_token);
            }
            SubModule::SubModule { .. } => {}
        }
    }

    Err(import_error(
        path_token,
        format!("cannot find import path `{}`", path_token.get_span().text()),
    ))
}

fn find_from_module<'a>(
    compiler: &'a Compiler,
    parts: &[String],
    path_token: &Token,
) -> Result<&'a SourceFile, ParserError> {
    let Some((module_name, rest)) = parts.split_first() else {
        return Err(import_error(path_token, "import path is empty"));
    };

    if module_name == "this" {
        if rest.is_empty() {
            return Err(import_error(path_token, "missing target after `this/`"));
        }
        return find_in_submodules(&compiler.compiling.files, rest, path_token);
    }

    let module = compiler
        .mods
        .iter()
        .find(|module| module.name == *module_name)
        .ok_or_else(|| import_error(path_token, format!("cannot find module `{module_name}`")))?;

    if rest.is_empty() {
        return Err(import_error(
            path_token,
            format!("missing target after module `{module_name}`"),
        ));
    }

    find_in_submodules(&module.files, rest, path_token)
}

fn parser_path(
    symtbl: &mut SymbolTable,
    path_token: &Token,
    path: &String,
) -> Result<Box<SourceFile>, ParserError> {
    let normalized_path = Compiler::normalize_path(path.as_str());
    let file = find_from_module(symtbl.compiler(), &normalized_path, path_token)?;
    Ok(Box::new(file.clone()))
}

pub fn import_parser(
    parser: &mut Parser,
    symtbl: &mut SymbolTable,
) -> Result<AstNode, ParserError> {
    let mut token = parser.get_token()?;
    if !matches!(token.get_type(), TokenType::Identifier) {
        return Err(ParserError::ExpectedToken(token, TokenType::Identifier));
    }
    let imp_name = token;
    token = parser.get_token()?;
    if !matches!(token.get_type(), TokenType::From) {
        return Err(ParserError::ExpectedToken(token, TokenType::From));
    }
    token = parser.get_token()?;

    let path_token = token.clone();
    if let TokenType::String(path) = path_token.get_type() {
        Ok(AstNode::Import {
            name: imp_name,
            file: parser_path(symtbl, &path_token, path)?,
        })
    } else {
        Err(ParserError::ExpectedToken(
            token,
            TokenType::String("lib_path".to_string()),
        ))
    }
}
