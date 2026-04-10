use std::collections::HashMap;

use super::lexer::Token;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IntTy {
    I8,
    I16,
    I32, // default
    I64,
    U8,
    U16,
    U32,
    U64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FltTy {
    F32, // default
    F64,
}

#[derive(Debug, Clone)]
pub enum TypeKind<'tcx> {
    Int(IntTy),
    Float(FltTy),
    Str,
    Null,
    Void,

    Tuple(&'tcx [TypeKind<'tcx>]),

    Enum {
        name: Token,
        element: Vec<(Token, Vec<TypeKind<'tcx>>)>,
    },
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

impl TypeKind<'_> {
    pub fn check(name: &str) -> Option<Self> {
        match name {
            "i8" => Some(TypeKind::Int(IntTy::I8)),
            "i16" => Some(TypeKind::Int(IntTy::I16)),
            "i32" => Some(TypeKind::Int(IntTy::I32)),
            "i64" => Some(TypeKind::Int(IntTy::I64)),
            "u8" => Some(TypeKind::Int(IntTy::U8)),
            "u16" => Some(TypeKind::Int(IntTy::U16)),
            "u32" => Some(TypeKind::Int(IntTy::U32)),
            "u64" => Some(TypeKind::Int(IntTy::U64)),
            "f32" => Some(TypeKind::Float(FltTy::F32)),
            "f64" => Some(TypeKind::Float(FltTy::F64)),
            "string" => Some(TypeKind::Str),
            "null" => Some(TypeKind::Null),
            _ => None,
        }
    }
}
