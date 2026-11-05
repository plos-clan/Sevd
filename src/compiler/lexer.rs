use crate::compiler::SourceFile;
use crate::compiler::com_error::LexError;
use crate::compiler::lexer::TokenType::Operator;
use line_column::span::{Span, TextRange, TextSize};
use std::fmt::Formatter;
use std::string::ToString;
use std::{fmt::Display, ops::Sub};
use text_size::TextLen;

const KEYWORDS: [(&str, TokenType); 20] = [
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
    ("struct", TokenType::Struct),
    ("restrict", TokenType::Restrict),
    ("as", TokenType::As),
];

#[derive(Debug)]
pub struct LexerAnalysis<'a> {
    file: &'a SourceFile,
    cache: Option<char>,
    pos: TextSize,
}

struct NumberLiteral {
    text: String,
    suffix: String,
    has_dot: bool,
    start: TextSize,
    // Lookahead can advance the cursor beyond the literal's source span.
    end: TextSize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenNumber {
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    F32(f32),
    F64(f64),
}

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq, PartialOrd, Ord)]
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
    Path,      // ::
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

impl Display for OperatorEnum {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            OperatorEnum::Ref => write!(f, "."),
            OperatorEnum::Set => write!(f, "="),
            OperatorEnum::NotEq => write!(f, "!="),
            OperatorEnum::Eq => write!(f, "=="),
            OperatorEnum::BigEq => write!(f, ">="),
            OperatorEnum::LesEq => write!(f, "<="),
            OperatorEnum::Big => write!(f, ">"),
            OperatorEnum::Less => write!(f, "<"),
            OperatorEnum::Colon => write!(f, ":"),
            OperatorEnum::Path => write!(f, "::"),
            OperatorEnum::Question => write!(f, "?"),
            OperatorEnum::Comma => write!(f, ","),
            OperatorEnum::Not => write!(f, "!"),
            OperatorEnum::And => write!(f, "&&"),
            OperatorEnum::Or => write!(f, "||"),
            OperatorEnum::BitOr => write!(f, "|"),
            OperatorEnum::BitAnd => write!(f, "&"),
            OperatorEnum::BitXor => write!(f, "^"),
            OperatorEnum::BitLeft => write!(f, "<<"),
            OperatorEnum::BitRight => write!(f, ">>"),
            OperatorEnum::Plus => write!(f, "++"),
            OperatorEnum::Minus => write!(f, "--"),
            OperatorEnum::AddSet => write!(f, "+="),
            OperatorEnum::SubSet => write!(f, "-="),
            OperatorEnum::MulSet => write!(f, "*="),
            OperatorEnum::DivSet => write!(f, "/="),
            OperatorEnum::ModSet => write!(f, "%="),
            OperatorEnum::BitAndSet => write!(f, "&="),
            OperatorEnum::BitXorSet => write!(f, "^="),
            OperatorEnum::BitOrSet => write!(f, "|="),
            OperatorEnum::Add => write!(f, "+"),
            OperatorEnum::Sub => write!(f, "-"),
            OperatorEnum::Mul => write!(f, "*"),
            OperatorEnum::Div => write!(f, "/"),
            OperatorEnum::Mod => write!(f, "%"),
            OperatorEnum::Array => write!(f, "[array]"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Eof,
    Identifier,
    Number(TokenNumber),
    Operator(OperatorEnum),
    String(String),
    Char(char),
    End,
    At,
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
    Struct,
    Restrict,
    As,
}

impl Display for TokenType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenType::Eof => f.write_str("<eof>"),
            TokenType::Identifier => f.write_str("<identifier>"),
            TokenType::Number(token_number) => f.write_fmt(format_args!("number {token_number:?}")),
            Operator(operator_enum) => f.write_fmt(format_args!("operator {operator_enum:?}")),
            TokenType::String(need) => f.write_fmt(format_args!("\"{need}\"")),
            TokenType::Char(char) => f.write_fmt(format_args!("'{char:?}'")),
            TokenType::End => f.write_str(";"),
            TokenType::At => f.write_str("@"),
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
            TokenType::Struct => f.write_str("struct"),
            TokenType::Restrict => f.write_str("restrict"),
            TokenType::As => f.write_str("as"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    span: Span,
    t_type: TokenType,
}

impl Token {
    #[must_use]
    pub fn new(span: Span, t_type: TokenType) -> Self {
        Self { span, t_type }
    }

    #[must_use]
    pub fn no_span_new(types: TokenType) -> Self {
        Self {
            span: Span::new_full(String::new()),
            t_type: types,
        }
    }

    #[must_use]
    pub fn line_column(&self) -> (u32, u32) {
        (self.span.line(), self.span.column())
    }

    #[must_use]
    pub fn is_eof(&self) -> bool {
        self.t_type == TokenType::Eof
    }

    #[must_use]
    pub fn get_span(&self) -> &Span {
        &self.span
    }

    #[must_use]
    pub fn get_type(&self) -> &TokenType {
        &self.t_type
    }
}

impl<'a> LexerAnalysis<'a> {
    #[must_use]
    pub fn new(file: &'a SourceFile) -> LexerAnalysis<'a> {
        Self {
            file,
            cache: None,
            pos: TextSize::from(0),
        }
    }

    fn match_keyword(identifier_name: &str) -> TokenType {
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
        let t_type = Self::match_keyword(span.text());
        Token { span, t_type }
    }

    fn number_error(&self, start: TextSize, end: TextSize, message: impl Into<String>) -> LexError {
        LexError::InvalidToken(
            message.into(),
            Token::new(
                self.make_span(start, end),
                TokenType::Number(TokenNumber::F32(0.0)),
            ),
        )
    }

    fn read_suffix(&mut self, prefix: char, start: TextSize) -> Result<String, LexError> {
        let kind = match prefix {
            'i' | 'u' => "integer",
            'f' => "float",
            _ => unreachable!(),
        };
        let mut suffix = String::from(prefix);
        let first = self.next_char();
        suffix.push(first);

        let valid = match (prefix, first) {
            ('i' | 'u', '8') => true,
            ('i' | 'u', '1' | '3' | '6') | ('f', '3' | '6') => {
                let second = self.next_char();
                suffix.push(second);
                matches!((first, second), ('1', '6') | ('3', '2') | ('6', '4'))
            }
            _ => false,
        };
        if !valid {
            return Err(self.number_error(
                start,
                self.pos,
                format!("unsupported {kind} suffix: {suffix}"),
            ));
        }
        Ok(suffix)
    }

    fn build_number(&mut self) -> Result<Token, LexError> {
        let first = self.next_char();
        let mut literal = NumberLiteral {
            text: String::from(first),
            suffix: String::new(),
            has_dot: first == '.',
            start: self.pos - first.text_len(),
            end: self.pos,
        };

        if first == '.' {
            let next = self.next_char();
            if !next.is_ascii_digit() {
                if next != '\0' {
                    self.cache = Some(next);
                }
                return Ok(Token::new(
                    self.make_span(literal.start, literal.end),
                    Operator(OperatorEnum::Ref),
                ));
            }
            literal.text.insert(0, '0');
            literal.text.push(next);
            literal.end = self.pos;
        }

        self.scan_number_tail(&mut literal)?;
        let number = Self::parse_number_value(&literal.text, &literal.suffix, literal.has_dot)
            .map_err(|message| self.number_error(literal.start, literal.end, message))?;
        Ok(Token::new(
            self.make_span(literal.start, literal.end),
            TokenType::Number(number),
        ))
    }

    fn scan_number_tail(&mut self, literal: &mut NumberLiteral) -> Result<(), LexError> {
        loop {
            let before = self.pos;
            let ch = self.next_char();
            match ch {
                '\0' => break,
                c if c.is_ascii_digit() => {
                    literal.text.push(c);
                    literal.end = self.pos;
                }
                '.' if !literal.has_dot => {
                    literal.has_dot = true;
                    literal.text.push('.');
                    literal.end = self.pos;
                }
                'i' | 'u' | 'f' => {
                    literal.suffix = self.read_suffix(ch, literal.start)?;
                    literal.end = self.pos;
                    break;
                }
                '_' => {
                    let next = self.next_char();
                    match next {
                        d if d.is_ascii_digit() => {
                            literal.text.push(d);
                            literal.end = self.pos;
                        }
                        'i' | 'u' | 'f' => {
                            literal.suffix = self.read_suffix(next, literal.start)?;
                            literal.end = self.pos;
                            break;
                        }
                        '\0' => break,
                        other => {
                            self.cache = Some(other);
                            literal.end = before;
                            break;
                        }
                    }
                }
                other => {
                    self.cache = Some(other);
                    literal.end = before;
                    break;
                }
            }
        }
        Ok(())
    }

    fn parse_number_value(text: &str, suffix: &str, has_dot: bool) -> Result<TokenNumber, String> {
        let number = if has_dot || suffix.starts_with('f') {
            match suffix {
                "" | "f32" => text
                    .parse::<f32>()
                    .map(TokenNumber::F32)
                    .map_err(|err| err.to_string()),
                "f64" => text
                    .parse::<f64>()
                    .map(TokenNumber::F64)
                    .map_err(|err| err.to_string()),
                _ => return Err(format!("unsupported float suffix: {suffix}")),
            }
        } else {
            match suffix {
                "" | "i32" => text
                    .parse::<i32>()
                    .map(TokenNumber::I32)
                    .map_err(|err| err.to_string()),
                "i8" => text
                    .parse::<i8>()
                    .map(TokenNumber::I8)
                    .map_err(|err| err.to_string()),
                "i16" => text
                    .parse::<i16>()
                    .map(TokenNumber::I16)
                    .map_err(|err| err.to_string()),
                "i64" => text
                    .parse::<i64>()
                    .map(TokenNumber::I64)
                    .map_err(|err| err.to_string()),
                "u8" => text
                    .parse::<u8>()
                    .map(TokenNumber::U8)
                    .map_err(|err| err.to_string()),
                "u16" => text
                    .parse::<u16>()
                    .map(TokenNumber::U16)
                    .map_err(|err| err.to_string()),
                "u32" => text
                    .parse::<u32>()
                    .map(TokenNumber::U32)
                    .map_err(|err| err.to_string()),
                "u64" => text
                    .parse::<u64>()
                    .map(TokenNumber::U64)
                    .map_err(|err| err.to_string()),
                _ => return Err(format!("unsupported integer suffix: {suffix}")),
            }
        };
        number.map_err(|err| format!("invalid numeric literal `{text}{suffix}`: {err}"))
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
    ) -> Token {
        let fist_char = self.next_char();
        match fist_char {
            '=' => Token {
                span: self.make_span(start, self.pos),
                t_type: Operator(one),
            },
            c if c == tchar => Token {
                span: self.make_span(start, self.pos),
                t_type: Operator(two),
            },
            c => {
                self.cache = Some(c);
                Token {
                    span: self.make_span(start, self.pos),
                    t_type: Operator(three),
                }
            }
        }
    }

    fn build_same(&mut self, start: TextSize, same: char) -> Token {
        let c = self.next_char();
        if c == same {
            Token {
                span: self.make_span(start, start),
                t_type: Operator(OperatorEnum::Path),
            }
        } else {
            self.cache = Some(c);
            Token {
                span: self.make_span(start, start),
                t_type: Operator(OperatorEnum::Colon),
            }
        }
    }

    fn build_two(&mut self, start: TextSize, fir: OperatorEnum, sec: OperatorEnum) -> Token {
        let fist_char = self.next_char();
        match fist_char {
            '=' => Token {
                span: self.make_span(start, self.pos),
                t_type: Operator(fir),
            },
            c => {
                self.cache = Some(c);
                Token {
                    span: self.make_span(start, self.pos),
                    t_type: Operator(sec),
                }
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
                        } else if c != '\0' {
                            continue;
                        }
                    } else if c != '\0' {
                        continue;
                    }
                    return Err(LexError::InvalidEof(Token {
                        span: self.make_span(start, self.pos),
                        t_type: TokenType::String(c.to_string()),
                    }));
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

    /// # Errors lex error
    fn build_char(&mut self, start: TextSize) -> Result<Token, LexError> {
        let mut c = self.next_char();
        let out_c = match c {
            '\\' => {
                c = self.next_char();
                match c {
                    '"' => '"',
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    '\\' => '\\',
                    '\'' => '\'',
                    c => {
                        return Err(LexError::InvalidToken(
                            format!("Illegal escape char: {c}"),
                            Token {
                                span: self.make_span(start, self.pos),
                                t_type: TokenType::Char(c),
                            },
                        ));
                    }
                }
            }
            c => c,
        };
        c = self.next_char();
        if c != '\'' {
            return Err(LexError::InvalidToken(
                "Expected '".to_string(),
                Token {
                    span: self.make_span(start, self.pos),
                    t_type: TokenType::Char(c),
                },
            ));
        }
        Ok(Token {
            span: self.make_span(start, self.pos),
            t_type: TokenType::Char(out_c),
        })
    }

    fn eliminate_empty_char(&mut self) -> (TextSize, char) {
        loop {
            let start_pos = self.pos;
            let start = self.next_char();
            if !matches!(start, '\n' | '\t' | ' ') {
                break (start_pos, start);
            }
        }
    }

    /// # Errors
    ///
    /// `Err` has unknown token
    pub fn get_token(&mut self) -> Result<Token, LexError> {
        let (start_pos, start) = self.eliminate_empty_char();

        match start {
            '\0' => Ok(Token {
                span: self.make_span(start_pos, self.pos),
                t_type: TokenType::Eof,
            }),
            ';' => Ok(Token {
                span: self.make_span(start_pos, self.pos),
                t_type: TokenType::End,
            }),
            '@' => Ok(Token {
                span: self.make_span(start_pos, self.pos),
                t_type: TokenType::At,
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
            ':' => Ok(self.build_same(start_pos, ':')),
            ',' => Ok(Token {
                span: self.make_span(start_pos, start_pos),
                t_type: Operator(OperatorEnum::Comma),
            }),
            '?' => Ok(Token {
                span: self.make_span(start_pos, start_pos),
                t_type: Operator(OperatorEnum::Question),
            }),
            '=' => Ok(self.build_two(start_pos, OperatorEnum::Eq, OperatorEnum::Set)),
            '+' => Ok(self.build_three(
                start_pos,
                '+',
                OperatorEnum::AddSet,
                OperatorEnum::Plus,
                OperatorEnum::Add,
            )),
            '-' => Ok(self.build_three(
                start_pos,
                '-',
                OperatorEnum::SubSet,
                OperatorEnum::Minus,
                OperatorEnum::Sub,
            )),
            '&' => Ok(self.build_three(
                start_pos,
                '&',
                OperatorEnum::BitAndSet,
                OperatorEnum::And,
                OperatorEnum::BitAnd,
            )),
            '|' => Ok(self.build_three(
                start_pos,
                '|',
                OperatorEnum::BitOrSet,
                OperatorEnum::Or,
                OperatorEnum::BitOr,
            )),
            '>' => Ok(self.build_three(
                start_pos,
                '>',
                OperatorEnum::BigEq,
                OperatorEnum::BitRight,
                OperatorEnum::Big,
            )),
            '<' => Ok(self.build_three(
                start_pos,
                '>',
                OperatorEnum::LesEq,
                OperatorEnum::BitLeft,
                OperatorEnum::Less,
            )),
            '\'' => self.build_char(start_pos),
            '^' => Ok(self.build_two(start_pos, OperatorEnum::BitXorSet, OperatorEnum::BitXor)),
            '!' => Ok(self.build_two(start_pos, OperatorEnum::NotEq, OperatorEnum::Not)),
            '*' => Ok(self.build_two(start_pos, OperatorEnum::MulSet, OperatorEnum::Mul)),
            '%' => Ok(self.build_two(start_pos, OperatorEnum::ModSet, OperatorEnum::Mod)),
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
