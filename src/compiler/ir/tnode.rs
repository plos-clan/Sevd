use crate::compiler::lexer::Token;
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
pub enum NumberKind {
    I8,
    I16,
    I32,
    I64,

    U8,
    U16,
    U32,
    U64,

    F32,
    F64,

    Signed,   // 有符号
    Unsigned, // 无符号

    // 任意数值类型都可匹配
    Number,
}

#[derive(Debug, Clone)]
pub enum PrimitiveType {
    Void,
    Bool,
    Number(NumberKind),

    // 因发生语义分析错误而拒绝推导的类型
    Error,
    Nothing,
}

impl PrimitiveType {
    #[must_use]
    pub fn cast(name: &str) -> Option<Self> {
        match name {
            "void" => Some(PrimitiveType::Void),
            "bool" => Some(PrimitiveType::Bool),
            "i8" => Some(PrimitiveType::Number(NumberKind::I8)),
            "i16" => Some(PrimitiveType::Number(NumberKind::I16)),
            "i32" => Some(PrimitiveType::Number(NumberKind::I32)),
            "i64" => Some(PrimitiveType::Number(NumberKind::I64)),
            "u8" => Some(PrimitiveType::Number(NumberKind::U8)),
            "u16" => Some(PrimitiveType::Number(NumberKind::U16)),
            "u32" => Some(PrimitiveType::Number(NumberKind::U32)),
            "u64" => Some(PrimitiveType::Number(NumberKind::U64)),
            "f32" => Some(PrimitiveType::Number(NumberKind::F32)),
            "f64" => Some(PrimitiveType::Number(NumberKind::F64)),
            _ => None,
        }
    }
}

impl Display for PrimitiveType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PrimitiveType::Void => write!(f, "void"),
            PrimitiveType::Bool => write!(f, "bool"),
            PrimitiveType::Number(NumberKind::I8) => write!(f, "i8"),
            PrimitiveType::Number(NumberKind::I16) => write!(f, "i16"),
            PrimitiveType::Number(NumberKind::I32) => write!(f, "i32"),
            PrimitiveType::Number(NumberKind::I64) => write!(f, "i64"),
            PrimitiveType::Number(NumberKind::U8) => write!(f, "u8"),
            PrimitiveType::Number(NumberKind::U16) => write!(f, "u16"),
            PrimitiveType::Number(NumberKind::U32) => write!(f, "u32"),
            PrimitiveType::Number(NumberKind::U64) => write!(f, "u64"),
            PrimitiveType::Number(NumberKind::F32) => write!(f, "f32"),
            PrimitiveType::Number(NumberKind::F64) => write!(f, "f64"),
            PrimitiveType::Error => write!(f, "error"),
            PrimitiveType::Nothing => write!(f, "nothing"),
            PrimitiveType::Number(NumberKind::Signed) => write!(f, "signed"),
            PrimitiveType::Number(NumberKind::Unsigned) => write!(f, "unsigned"),
            PrimitiveType::Number(NumberKind::Number) => write!(f, "number"),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct GenericParam {
    name: Token,
    bounds: Vec<PrimitiveType>, // 泛型约束
}
