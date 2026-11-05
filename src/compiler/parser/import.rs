use std::path::Path;

use crate::compiler::com_error::ParserError;
use crate::compiler::ir::AstNode;
use crate::compiler::lexer::TokenType;
use crate::compiler::lexer::{OperatorEnum, Token};
use crate::compiler::module::ModuleId;
use crate::compiler::symtbl::SymbolTable;

use super::Parser;

fn import_error(token: &Token, message: impl Into<String>) -> ParserError {
    ParserError::UnknownLibrary(token.clone(), message.into())
}

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

fn parser_path(
    symtbl: &SymbolTable,
    path_token: &Token,
    path: &str,
) -> Result<ModuleId, ParserError> {
    let parts = import_module_path(path)
        .ok_or_else(|| import_error(path_token, format!("invalid module path `{path}`")))?;
    let module = symtbl
        .compiler()
        .resolve_module(&parts)
        .ok_or_else(|| import_error(path_token, format!("cannot find module `{path}`")))?;
    if module.file.is_none() {
        return Err(import_error(
            path_token,
            format!("module `{path}` does not have a source file"),
        ));
    }
    Ok(module.id)
}

fn named_parse(parser: &mut Parser) -> Result<Vec<Token>, ParserError> {
    let mut token = parser.get_token()?;
    match token.get_type() {
        TokenType::Identifier => Ok(vec![token]),
        TokenType::Lp('{') => {
            let mut named = vec![];
            loop {
                token = parser.get_token()?;
                let TokenType::Identifier = token.get_type() else {
                    return Err(ParserError::ExpectedToken(token, TokenType::Identifier));
                };
                named.push(token);
                token = parser.get_token()?;
                match token.get_type() {
                    TokenType::Lr('}') => break,
                    TokenType::Operator(OperatorEnum::Comma) => {}
                    _ => return Err(ParserError::ExpectedToken(token, TokenType::Lp('}'))),
                }
            }
            Ok(named)
        }
        _ => Err(ParserError::ExpectedToken(token, TokenType::Identifier)),
    }
}

pub fn import_parser(
    parser: &mut Parser,
    symtbl: &mut SymbolTable,
) -> Result<AstNode, ParserError> {
    let imp_name = named_parse(parser)?;

    let mut token = parser.get_token()?;
    if !matches!(token.get_type(), TokenType::From) {
        return Err(ParserError::ExpectedToken(token, TokenType::From));
    }
    token = parser.get_token()?;

    let path_token = token.clone();
    if let TokenType::String(path) = path_token.get_type() {
        Ok(AstNode::Import {
            name: imp_name,
            module_id: parser_path(symtbl, &path_token, path)?,
        })
    } else {
        Err(ParserError::ExpectedToken(
            token,
            TokenType::String("lib_path".to_string()),
        ))
    }
}
