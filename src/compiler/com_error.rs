use super::lexer::{OperatorEnum, Token, TokenType};
use crate::compiler::SourceFile;
use crate::compiler::ir::tnode::TypeKind;
use std::io::IsTerminal;

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
    IllegalStmtEnd(Token),           // 非法语句结束
    IllegalExpression(Token),        // 非法的表达式组合
    MissingCondition(Token),         // 缺少条件表达式
    IllegalKey(Token),               // 非法的关键字
    MissingFunctionBody(Token),      // 缺少函数体
    MissingLoopBody(Token),          // 缺少循环体
    MissingStatement(Token),         // 语句定义不完整
    MissingEnumElement(Token),       // 缺少枚举项
}

#[derive(Debug, Clone)]
pub enum SematicError {
    InvalidAnnotationTarget(Token),                    // 非法的注解绑定
    InvalidGeneratorPassing(Token),                    // 非法的泛型传递
    InvalidTypeApplied(Token, OperatorEnum, TypeKind), // 非法类型应用
    MissingGenericConstraint(Token),                   // 未提供泛型的绑定规则
    MissingInitializer(Token),                         // 未提供初始值
    MissingBlockFinalValue(Token),                     // 块语句缺失最终值
    ItemAlreadyDefined(Token),                         // 重复定义项
    UnresolvedSymbol(Token),                           // 无法解析符号
    InvalidBinding(Token),                             // 非法的绑定形式
}

fn print_infomation(file: &SourceFile, message: &str, token: &Token) {
    eprintln!("SyntaxError ({}): {message}", file.name);
    let start = token.get_span().start();
    let (line, column) = (start.line(), start.column());
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

    eprintln!("{line_prefix} | {line_text}");
    if is_terminal {
        eprintln!("{gutter_padding} | {indicator_padding}\x1b[31m{indicator}\x1b[0m");
    } else {
        eprintln!("{gutter_padding} | {indicator_padding}{indicator}");
    }
}

pub fn print_parser_error(file: &SourceFile, error: ParserError) {
    let (message, token) = match error {
        ParserError::LexError(e) => match e {
            LexError::UnknownChar(c, token) => (format!("Unrecognised character: {c}"), token),
            LexError::InvalidEof(token) => ("Invalid eof".to_string(), token),
            LexError::InvalidToken(msg, token) => (msg, token),
        },
        ParserError::NotAStatement(token) => ("Not a statement".to_string(), token),
        ParserError::Expected(token, c) => (format!("'{c}' expected."), token),
        ParserError::ExpectedToken(token, key) => (format!("'{key}' expected."), token),
        ParserError::UnknownLibrary(token, message) => (message, token),
        ParserError::IllegalStmtEnd(token) => ("Illegal statement end".to_string(), token),
        ParserError::IllegalExpression(token) => ("Illegal expression".to_string(), token),
        ParserError::MissingCondition(token) => ("Missing condition".to_string(), token),
        ParserError::IllegalKey(token) => ("Illegal key".to_string(), token),
        ParserError::MissingFunctionBody(token) => ("Missing function body".to_string(), token),
        ParserError::MissingLoopBody(token) => ("Missing loop body".to_string(), token),
        ParserError::MissingStatement(token) => ("Missing statement".to_string(), token),
        ParserError::MissingEnumElement(token) => ("Missing enum element".to_string(), token),
    };
    print_infomation(file, message.as_str(), &token);
}

fn print_unit_error(file: &SourceFile, error: SematicError) {
    let (message, token) = match error {
        SematicError::InvalidAnnotationTarget(token) => {
            ("Invalid annotation target".to_string(), token)
        }
        SematicError::MissingInitializer(token) => ("Missing initializer".to_string(), token),
        SematicError::MissingGenericConstraint(token) => {
            ("Missing generic constraint".to_string(), token)
        }
        SematicError::MissingBlockFinalValue(token) => {
            ("Missing block final value".to_string(), token)
        }
        SematicError::UnresolvedSymbol(token) => ("Unresolved symbol".to_string(), token),
        SematicError::InvalidGeneratorPassing(token) => {
            ("Invalid generator passing".to_string(), token)
        }
        SematicError::InvalidTypeApplied(token, opt, kind) => (
            format!("Operator '{opt}' cannot be applied to type '{kind}'").to_string(),
            token,
        ),
        SematicError::ItemAlreadyDefined(token) => (
            format!(
                "The name `{}` is defined multiple times",
                token.get_span().text()
            )
            .to_string(),
            token,
        ),
        SematicError::InvalidBinding(token) => ("invalid pattern binding".to_string(), token),
    };
    print_infomation(file, message.as_str(), &token);
}

pub fn print_semantic_error(file: &SourceFile, error: Vec<SematicError>) {
    for error in error {
        print_unit_error(file, error);
    }
}
