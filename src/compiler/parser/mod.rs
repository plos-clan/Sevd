mod annotation;
pub mod block;
mod define;
mod expr;
mod fors;
mod function;
mod generics;
mod guard;
mod ifs;
mod import;
mod pattern;
mod whiles;

use super::ir::AstNode;
use super::symtbl::SymbolTable;
use crate::compiler::com_error::ParserError;
use crate::compiler::com_error::ParserError::LexError;
use crate::compiler::lexer::Token;
use crate::compiler::lexer::{LexerAnalysis, TokenType};
use crate::compiler::parser::annotation::annotation_parser;
use crate::compiler::parser::define::{enum_parser, struct_parser, var_parser};
use crate::compiler::parser::function::function_parser;
use crate::compiler::parser::import::import_parser;
use std::iter::Peekable;
use std::vec::IntoIter;

pub struct Parser<'a> {
    lex: Option<&'a mut LexerAnalysis<'a>>,
    tokens: Peekable<IntoIter<Token>>,
    pub cache: Option<Token>,
}

impl<'a> Parser<'a> {
    pub fn new(lex: &'a mut LexerAnalysis<'a>) -> Self {
        Self {
            lex: Some(lex),
            cache: None,
            tokens: vec![].into_iter().peekable(),
        }
    }

    pub fn new_collect(tokens: Vec<Token>) -> Self {
        Self {
            lex: None,
            cache: None,
            tokens: tokens.into_iter().peekable(),
        }
    }

    fn get_token(&mut self) -> Result<Token, ParserError> {
        if let Some(lex) = self.lex.as_mut() {
            let Some(token) = self.cache.take() else {
                return match lex.get_token() {
                    Ok(token) => Ok(token),
                    Err(err) => Err(LexError(err)),
                };
            };
            self.cache = None;
            Ok(token)
        } else {
            Ok(self
                .cache
                .take()
                .unwrap_or_else(|| match self.tokens.peek().cloned() {
                    Some(token) => {
                        self.tokens.next();
                        token
                    }
                    None => Token::no_span_new(TokenType::Eof),
                }))
        }
    }

    fn root_parser(
        &mut self,
        symtbl: &mut SymbolTable<'_>,
    ) -> Result<Option<AstNode>, ParserError> {
        let token = self.get_token()?;
        if token.is_eof() {
            return Ok(None);
        }

        match token.get_type() {
            TokenType::Import => Ok(Some(import_parser(self, symtbl)?)),
            TokenType::Function => Ok(Some(function_parser(self)?)),
            TokenType::Let => Ok(Some(var_parser(self, token)?)),
            TokenType::Enum => Ok(Some(enum_parser(self)?)),
            TokenType::Struct => Ok(Some(struct_parser(self)?)),
            TokenType::At => Ok(Some(annotation_parser(self)?)),
            TokenType::End => Ok(self.root_parser(symtbl)?),
            _ => Err(ParserError::NotAStatement(token)),
        }
    }

    pub fn parser(&mut self, symtbl: &mut SymbolTable<'_>) -> Result<Vec<AstNode>, ParserError> {
        let mut nodes = vec![];
        while let Some(node) = self.root_parser(symtbl)? {
            nodes.push(node);
        }
        Ok(nodes)
    }
}
