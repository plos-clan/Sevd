use std::io::IsTerminal;

use super::lexer::{Token, TokenType};
use crate::compiler::SourceFile;

#[derive(Debug, Clone)]
pub enum LexError {
    UnknownChar(char, Token),    // 未知字符
    InvalidToken(String, Token), // 非法词素
    InvalidEof(Token),           // 非法终止组合
}

#[derive(Debug, Clone)]
pub enum ParserError {
    LexError(LexError),              // 词法分析错误
    NotAStatement(Token),            // 不是一个语句
    Expected(Token, char),           // 需要指定字符
    ExpectedToken(Token, TokenType), // 需要制定词元类型
    UnknownLibrary(Token, String),   // 找不到指定的模块
    UnknownType(Token),              // 未知类型
    IllegalExpression(Token),        // 非法的表达式组合
    MissingCondition(Token),         // 缺少条件表达式
    IllegalKey(Token),               // 非法的关键字
    MissingFunctionBody(Token),      // 缺少函数体
    MissingLoopBody(Token),          // 缺少循环体
    MissingStatement(Token),         // 语句定义不完整
    MissingEnumElement(Token),       // 缺少枚举项
}

pub fn print_parser_error(file: &SourceFile, error: ParserError) {
    let (message, token) = match error {
        ParserError::LexError(e) => match e {
            LexError::UnknownChar(c, token) => (format!("Unrecognised character: {}", c), token),
            LexError::InvalidEof(token) => ("Invalid eof".to_string(), token),
            LexError::InvalidToken(msg, token) => (msg, token),
        },
        ParserError::NotAStatement(token) => ("Not a statement".to_string(), token),
        ParserError::Expected(token, c) => (format!("'{}' expected.", c), token),
        ParserError::ExpectedToken(token, key) => (format!("'{}' expected.", key), token),
        ParserError::UnknownLibrary(token, message) => (message, token),
        ParserError::UnknownType(token) => ("Unknown type".to_string(), token),
        ParserError::IllegalExpression(token) => ("Illegal expression".to_string(), token),
        ParserError::MissingCondition(token) => ("Missing condition".to_string(), token),
        ParserError::IllegalKey(token) => ("Illegal key".to_string(), token),
        ParserError::MissingFunctionBody(token) => ("Missing function body".to_string(), token),
        ParserError::MissingLoopBody(token) => ("Missing loop body".to_string(), token),
        ParserError::MissingStatement(token) => ("Missing statement".to_string(), token),
        ParserError::MissingEnumElement(token) => ("Missing enum element".to_string(), token),
    };
    eprintln!("SyntaxError ({}): {message}", file.name);

    let start = token.get_span().start();
    let (line, column) = start.line_column();
    let line_number = line.max(1) as usize;
    let line_index = line.saturating_sub(1) as usize;
    let line_text = file.data.source().lines().nth(line_index).unwrap_or("");
    let column_index = column.saturating_sub(1) as usize;
    let line_chars = line_text.chars().count();
    let pointer_offset = column_index.min(line_chars);
    let token_len = token.get_span().text().chars().count();
    let underline_len = token_len
        .max(1)
        .min(line_chars.saturating_sub(pointer_offset).max(1));

    let is_terminal = std::io::stderr().is_terminal();
    let line_prefix = line_number.to_string();
    let gutter_padding = " ".repeat(line_prefix.len());
    let indicator_padding = " ".repeat(pointer_offset);
    let indicator = format!("^{}", "~".repeat(underline_len.saturating_sub(1)));

    eprintln!("{} | {}", line_prefix, line_text);
    if is_terminal {
        eprintln!(
            "{} | {}\x1b[31m{}\x1b[0m",
            gutter_padding, indicator_padding, indicator
        );
    } else {
        eprintln!("{} | {}{}", gutter_padding, indicator_padding, indicator);
    }
}
