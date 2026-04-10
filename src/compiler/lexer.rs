use crate::compiler::SourceFile;
use crate::compiler::com_error::LexError;
use crate::compiler::lexer::TokenType::Operator;
use crate::compiler::typedef::TokenNumber;
use line_column::span::{Span, TextRange, TextSize};
use std::fmt::Formatter;
use std::string::ToString;
use std::{fmt::Display, ops::Sub};
use text_size::TextLen;

const KEYWORDS: [(&str, TokenType); 17] = [
    ("for", TokenType::For),
    ("while", TokenType::While),
    ("if", TokenType::If),
    ("elif", TokenType::Elif),
    ("else", TokenType::Else),
    ("return", TokenType::Return),
    ("break", TokenType::Break),
    ("continue", TokenType::Continue),
    ("import", TokenType::Import),
    ("function", TokenType::Function),
    ("true", TokenType::True),
    ("false", TokenType::False),
    ("let", TokenType::Let),
    ("null", TokenType::Null),
    ("export", TokenType::Export),
    ("from", TokenType::From),
    ("enum", TokenType::Enum),
];

#[derive(Debug)]
pub struct LexerAnalysis<'a> {
    file: &'a SourceFile,
    cache: Option<char>,
    pos: TextSize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OperatorEnum {
    Ref,       // .
    Set,       // =
    NotEq,     // !=
    Eq,        // ==
    BigEq,     // >=
    LesEq,     // <=
    Big,       // >
    Less,      // <
    Colon,     // :
    Question,  // ?
    Comma,     // ,
    Not,       // !
    And,       // &&
    Or,        // ||
    BitOr,     // |
    BitAnd,    // &
    BitXor,    // ^
    BitLeft,   // <<
    BitRight,  // >>
    Plus,      // ++
    Minus,     // --
    AddSet,    // +=
    SubSet,    // -=
    MulSet,    // *=
    DivSet,    // /=
    ModSet,    // %=
    BitAndSet, // &=
    BitXorSet, // ^=
    BitOrSet,  // |=
    Add,       // +
    Sub,       // -
    Mul,       // *
    Div,       // /
    Mod,       // %

    Array, // [] (语法分析填充)
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Eof,
    Identifier,
    Number(TokenNumber),
    Operator(OperatorEnum),
    String(String),
    End,
    Lp(char),
    Lr(char),

    For,
    If,
    Else,
    Function,
    Let,
    While,
    From,
    Elif,
    True,
    False,
    Return,
    Break,
    Continue,
    Import,
    Null,
    Export,
    Enum,
}

impl Display for TokenType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenType::Eof => f.write_str("<eof>"),
            TokenType::Identifier => f.write_str("<identifier>"),
            TokenType::Number(token_number) => {
                f.write_fmt(format_args!("number {:?}", token_number))
            }
            Operator(operator_enum) => f.write_fmt(format_args!("operator {:?}", operator_enum)),
            TokenType::String(need) => f.write_fmt(format_args!("\"{need}\"")),
            TokenType::End => f.write_str(";"),
            TokenType::Lp(c) | TokenType::Lr(c) => f.write_fmt(format_args!("{c}")),
            TokenType::For => f.write_str("for"),
            TokenType::If => f.write_str("if"),
            TokenType::Else => f.write_str("else"),
            TokenType::Function => f.write_str("function"),
            TokenType::Let => f.write_str("let"),
            TokenType::While => f.write_str("while"),
            TokenType::From => f.write_str("from"),
            TokenType::Elif => f.write_str("elif"),
            TokenType::True => f.write_str("true"),
            TokenType::False => f.write_str("false"),
            TokenType::Return => f.write_str("return"),
            TokenType::Break => f.write_str("break"),
            TokenType::Continue => f.write_str("continue"),
            TokenType::Import => f.write_str("import"),
            TokenType::Null => f.write_str("null"),
            TokenType::Export => f.write_str("export"),
            TokenType::Enum => f.write_str("enum"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    span: Span,
    t_type: TokenType,
}

impl Token {
    pub fn no_span_new(types: TokenType) -> Self {
        Self {
            span: Span::new_full("".to_string()),
            t_type: types,
        }
    }

    pub fn line_column(&self) -> (u32, u32) {
        self.span.line_column()
    }

    pub fn is_eof(&self) -> bool {
        self.t_type == TokenType::Eof
    }

    pub fn get_span(&self) -> &Span {
        &self.span
    }

    pub fn get_type(&self) -> &TokenType {
        &self.t_type
    }
}

impl<'a> LexerAnalysis<'a> {
    pub fn new(file: &'a SourceFile) -> LexerAnalysis<'a> {
        Self {
            file,
            cache: None,
            pos: TextSize::from(0),
        }
    }

    fn match_keyword(&self, identifier_name: &str) -> TokenType {
        for (keyword, token_type) in &KEYWORDS {
            if identifier_name == *keyword {
                return token_type.clone();
            }
        }
        TokenType::Identifier
    }

    fn next_char(&mut self) -> char {
        if let Some(cached) = self.cache.take() {
            return cached;
        }
        let remaining = &self.file.data.text()[self.pos.into()..];
        let ch = remaining.chars().next().unwrap_or('\0');
        if ch != '\0' {
            self.pos += ch.text_len();
        }
        ch
    }

    fn build_identifier(&mut self) -> Token {
        let start = self.pos.sub(TextSize::from(1));
        let mut text = String::new();
        let mut end = self.pos;
        loop {
            match self.next_char() {
                c if c.is_alphabetic() || c == '_' || c.is_ascii_digit() => {
                    text.push(c);
                    end = self.pos;
                }
                c => {
                    self.cache = Some(c);
                    break;
                }
            }
        }
        let span = self.make_span(start, end);
        let t_type = self.match_keyword(span.text());
        Token { span, t_type }
    }

    fn read_suffix(&mut self, prefix: char, start: TextSize) -> Result<String, LexError> {
        let mut suffix = String::new();
        suffix.push(prefix);

        match prefix {
            'i' | 'u' => {
                let first = self.next_char();
                suffix.push(first);

                match first {
                    '8' => {}
                    '1' | '3' | '6' => {
                        let second = self.next_char();
                        suffix.push(second);

                        match (first, second) {
                            ('1', '6') | ('3', '2') | ('6', '4') => {}
                            _ => {
                                return Err(LexError::InvalidToken(
                                    format!("unsupported integer suffix: {suffix}"),
                                    Token {
                                        span: self.make_span(start, self.pos),
                                        t_type: TokenType::Number(TokenNumber::F32(0.0)),
                                    },
                                ));
                            }
                        }
                    }
                    _ => {
                        return Err(LexError::InvalidToken(
                            format!("unsupported integer suffix: {suffix}"),
                            Token {
                                span: self.make_span(start, self.pos),
                                t_type: TokenType::Number(TokenNumber::F32(0.0)),
                            },
                        ));
                    }
                }
            }
            'f' => {
                let first = self.next_char();
                suffix.push(first);

                match first {
                    '3' => {
                        let second = self.next_char();
                        suffix.push(second);
                        if second != '2' {
                            return Err(LexError::InvalidToken(
                                format!("unsupported float suffix: {suffix}"),
                                Token {
                                    span: self.make_span(start, self.pos),
                                    t_type: TokenType::Number(TokenNumber::F32(0.0)),
                                },
                            ));
                        }
                    }
                    '6' => {
                        let second = self.next_char();
                        suffix.push(second);
                        if second != '4' {
                            return Err(LexError::InvalidToken(
                                format!("unsupported float suffix: {suffix}"),
                                Token {
                                    span: self.make_span(start, self.pos),
                                    t_type: TokenType::Number(TokenNumber::F32(0.0)),
                                },
                            ));
                        }
                    }
                    _ => {
                        return Err(LexError::InvalidToken(
                            format!("unsupported float suffix: {suffix}"),
                            Token {
                                span: self.make_span(start, self.pos),
                                t_type: TokenType::Number(TokenNumber::F32(0.0)),
                            },
                        ));
                    }
                }
            }
            _ => unreachable!(),
        }

        Ok(suffix)
    }

    fn build_number(&mut self) -> Result<Token, LexError> {
        let first_char = self.next_char();
        let start = self.pos - first_char.text_len();
        let mut end = self.pos;
        let mut literal = String::new();
        let mut suffix = String::new();
        let mut saw_dot = false;

        if first_char == '.' {
            let next = self.next_char();
            if !next.is_ascii_digit() {
                if next != '\0' {
                    self.cache = Some(next);
                }
                return Ok(Token {
                    span: self.make_span(start, start + first_char.text_len()),
                    t_type: Operator(OperatorEnum::Ref),
                });
            }

            saw_dot = true;
            literal.push('0');
            literal.push('.');
            literal.push(next);
            end = self.pos;
        } else {
            literal.push(first_char);
        }

        loop {
            let before = self.pos;
            let ch = self.next_char();

            match ch {
                '\0' => break,
                c if c.is_ascii_digit() => {
                    literal.push(c);
                    end = self.pos;
                }
                '.' if !saw_dot => {
                    saw_dot = true;
                    literal.push('.');
                    end = self.pos;
                }
                'i' | 'u' | 'f' => {
                    suffix = self.read_suffix(ch, start)?;
                    end = self.pos;
                    break;
                }
                '_' => {
                    let next = self.next_char();
                    match next {
                        d if d.is_ascii_digit() => {
                            literal.push(d);
                            end = self.pos;
                        }
                        'i' | 'u' | 'f' => {
                            suffix = self.read_suffix(next, start)?;
                            end = self.pos;
                            break;
                        }
                        '\0' => break,
                        other => {
                            self.cache = Some(other);
                            end = before;
                            break;
                        }
                    }
                }
                other => {
                    self.cache = Some(other);
                    end = before;
                    break;
                }
            }
        }

        let is_float = saw_dot || suffix.starts_with('f');
        let number = if is_float {
            match suffix.as_str() {
                "" | "f32" => TokenNumber::F32(literal.parse::<f32>().unwrap()),
                "f64" => TokenNumber::F64(literal.parse::<f64>().unwrap()),
                _ => {
                    return Err(LexError::InvalidToken(
                        format!("unsupported float suffix: {suffix}"),
                        Token {
                            span: self.make_span(start, self.pos),
                            t_type: TokenType::Number(TokenNumber::F32(0.0)),
                        },
                    ));
                }
            }
        } else {
            match suffix.as_str() {
                "" | "i32" => TokenNumber::I32(literal.parse::<i32>().unwrap()),
                "i8" => TokenNumber::I8(literal.parse::<i8>().unwrap()),
                "i16" => TokenNumber::I16(literal.parse::<i16>().unwrap()),
                "i64" => TokenNumber::I64(literal.parse::<i64>().unwrap()),
                "u8" => TokenNumber::U8(literal.parse::<u8>().unwrap()),
                "u16" => TokenNumber::U16(literal.parse::<u16>().unwrap()),
                "u32" => TokenNumber::U32(literal.parse::<u32>().unwrap()),
                "u64" => TokenNumber::U64(literal.parse::<u64>().unwrap()),
                _ => {
                    return Err(LexError::InvalidToken(
                        format!("unsupported integer suffix: {suffix}"),
                        Token {
                            span: self.make_span(start, self.pos),
                            t_type: TokenType::Number(TokenNumber::F32(0.0)),
                        },
                    ));
                }
            }
        };

        Ok(Token {
            span: self.make_span(start, end),
            t_type: TokenType::Number(number),
        })
    }

    fn build_string(&mut self, start: TextSize) -> Result<Token, LexError> {
        let mut raw_str = String::new();
        loop {
            let mut ch = self.next_char();
            match ch {
                '\\' => {
                    ch = self.next_char();
                    match ch {
                        '"' => raw_str.push('"'),
                        'n' => raw_str.push('\n'),
                        'r' => raw_str.push('\r'),
                        't' => raw_str.push('\t'),
                        '\\' => raw_str.push('\\'),
                        '\'' => raw_str.push('\''),
                        c => {
                            return Err(LexError::InvalidToken(
                                format!("Illegal escape char: {c}"),
                                Token {
                                    span: self.make_span(start, self.pos),
                                    t_type: TokenType::String(raw_str),
                                },
                            ));
                        }
                    }
                }
                '\0' => {
                    return Err(LexError::InvalidToken(
                        "'\"' expected.".to_string(),
                        Token {
                            span: self.make_span(start, self.pos),
                            t_type: TokenType::String(raw_str),
                        },
                    ));
                }
                '"' => break,
                c => raw_str.push(c),
            }
        }
        Ok(Token {
            span: self.make_span(start, self.pos),
            t_type: TokenType::String(raw_str),
        })
    }

    fn build_three(
        &mut self,
        start: TextSize,
        tchar: char,
        one: OperatorEnum,
        two: OperatorEnum,
        three: OperatorEnum,
    ) -> Result<Token, LexError> {
        let fist_char = self.next_char();
        match fist_char {
            '=' => Ok(Token {
                span: self.make_span(start, self.pos),
                t_type: Operator(one),
            }),
            c if c == tchar => Ok(Token {
                span: self.make_span(start, self.pos),
                t_type: Operator(two),
            }),
            c => {
                self.cache = Some(c);
                Ok(Token {
                    span: self.make_span(start, self.pos),
                    t_type: Operator(three),
                })
            }
        }
    }

    fn build_two(
        &mut self,
        start: TextSize,
        fir: OperatorEnum,
        sec: OperatorEnum,
    ) -> Result<Token, LexError> {
        let fist_char = self.next_char();
        match fist_char {
            '=' => Ok(Token {
                span: self.make_span(start, self.pos),
                t_type: Operator(fir),
            }),
            c => {
                self.cache = Some(c);
                Ok(Token {
                    span: self.make_span(start, self.pos),
                    t_type: Operator(sec),
                })
            }
        }
    }

    fn build_opt_skip_text(&mut self, start: TextSize) -> Result<Token, LexError> {
        let mut c = self.next_char();
        match c {
            '/' => {
                loop {
                    c = self.next_char();
                    if c == '\n' || c == '\0' {
                        break;
                    }
                }
                self.cache = Some(c);
                self.get_token()
            }
            '*' => {
                loop {
                    c = self.next_char();
                    if c == '*' {
                        c = self.next_char();
                        if c == '/' {
                            break;
                        } else if c == '\0' {
                            return Err(LexError::InvalidEof(Token {
                                span: self.make_span(start, self.pos),
                                t_type: TokenType::String(c.to_string()),
                            }));
                        }
                    } else if c == '\0' {
                        return Err(LexError::InvalidEof(Token {
                            span: self.make_span(start, self.pos),
                            t_type: TokenType::String(c.to_string()),
                        }));
                    }
                }
                self.get_token()
            }
            '=' => Ok(Token {
                span: self.make_span(start, self.pos),
                t_type: Operator(OperatorEnum::DivSet),
            }),
            c => {
                self.cache = Some(c);
                Ok(Token {
                    span: self.make_span(start, self.pos),
                    t_type: Operator(OperatorEnum::Div),
                })
            }
        }
    }

    pub fn get_token(&mut self) -> Result<Token, LexError> {
        let (start_pos, start) = loop {
            let start_pos = self.pos;
            let start = self.next_char();
            if !matches!(start, '\n' | '\t' | ' ') {
                break (start_pos, start);
            }
        };

        match start {
            '\0' => Ok(Token {
                span: self.make_span(start_pos, start_pos),
                t_type: TokenType::Eof,
            }),
            ';' => Ok(Token {
                span: self.make_span(start_pos, start_pos),
                t_type: TokenType::End,
            }),
            '"' => self.build_string(start_pos),
            ch if ch.is_alphabetic() || ch == '_' => {
                self.cache = Some(ch);
                Ok(self.build_identifier())
            }
            ch if ch.is_ascii_digit() || ch == '.' => {
                self.cache = Some(ch);
                self.build_number()
            }
            ':' => Ok(Token {
                span: self.make_span(start_pos, start_pos),
                t_type: Operator(OperatorEnum::Colon),
            }),
            ',' => Ok(Token {
                span: self.make_span(start_pos, start_pos),
                t_type: Operator(OperatorEnum::Comma),
            }),
            '?' => Ok(Token {
                span: self.make_span(start_pos, start_pos),
                t_type: Operator(OperatorEnum::Question),
            }),
            '=' => self.build_two(start_pos, OperatorEnum::Eq, OperatorEnum::Set),
            '+' => self.build_three(
                start_pos,
                '+',
                OperatorEnum::AddSet,
                OperatorEnum::Plus,
                OperatorEnum::Add,
            ),
            '-' => self.build_three(
                start_pos,
                '-',
                OperatorEnum::SubSet,
                OperatorEnum::Minus,
                OperatorEnum::Sub,
            ),
            '&' => self.build_three(
                start_pos,
                '&',
                OperatorEnum::BitAndSet,
                OperatorEnum::And,
                OperatorEnum::BitAnd,
            ),
            '|' => self.build_three(
                start_pos,
                '|',
                OperatorEnum::BitOrSet,
                OperatorEnum::Or,
                OperatorEnum::BitOr,
            ),
            '>' => self.build_three(
                start_pos,
                '>',
                OperatorEnum::BigEq,
                OperatorEnum::BitRight,
                OperatorEnum::Big,
            ),
            '<' => self.build_three(
                start_pos,
                '>',
                OperatorEnum::LesEq,
                OperatorEnum::BitLeft,
                OperatorEnum::Less,
            ),
            '^' => self.build_two(start_pos, OperatorEnum::BitXorSet, OperatorEnum::BitXor),
            '!' => self.build_two(start_pos, OperatorEnum::NotEq, OperatorEnum::Not),
            '*' => self.build_two(start_pos, OperatorEnum::MulSet, OperatorEnum::Mul),
            '%' => self.build_two(start_pos, OperatorEnum::ModSet, OperatorEnum::Mod),
            '/' => self.build_opt_skip_text(start_pos),
            c if matches!(c, '{' | '[' | '(') => Ok(Token {
                span: self.make_span(start_pos, start_pos),
                t_type: TokenType::Lp(c),
            }),
            c if matches!(c, '}' | ']' | ')') => Ok(Token {
                span: self.make_span(start_pos, start_pos),
                t_type: TokenType::Lr(c),
            }),
            c => Err(LexError::UnknownChar(
                c,
                Token {
                    span: self.make_span(start_pos, self.pos),
                    t_type: TokenType::Eof,
                },
            )),
        }
    }

    fn make_span(&self, start: TextSize, end: TextSize) -> Span {
        let range = TextRange::new(start, end);
        self.file.data.slice(range)
    }
}
