use std::fmt::Display;
use crate::compiler::lexer::Token;

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

    // 任意数值类型都可匹配
    Number,
}

#[derive(Debug, Clone)]
pub enum TypeKind {
    /// i32 解析为 {name: "i32", args: []}
    /// Result<_,i32> 解析为
    /// {name: "Result", args: [
    ///     Named { name: "i32" },
    ///     Infer,
    /// }
    Named {
        name: Token,
        args: Vec<TypeKind>,
    },
    Tuple(Vec<TypeKind>),
    Fn {
        params: Vec<TypeKind>,
        ret: Box<TypeKind>,
    },
    /// 空洞类型 _ 等第二次解析回填到 `HoleTable` 里
    Hole(u64),
    Void,
    Bool,
    Number(NumberKind),

    // 因发生语义分析错误而拒绝推导的类型
    Nothing,
}

impl Display for TypeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeKind::Named { name, args } => {
                write!(f, "{}", name.get_span().text())?;
                if !args.is_empty() {
                    write!(f, "<")?;
                    for (i, arg) in args.iter().enumerate() {
                        if i != 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{arg}")?;
                    }
                    write!(f, ">")?;
                }
                Ok(())
            }
            TypeKind::Tuple(items) => {
                write!(f, "(")?;
                for (i, item) in items.iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{item}")?;
                }
                write!(f, ")")
            }
            TypeKind::Fn { params, ret } => {
                write!(f, "fn(")?;
                for (i, param) in params.iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{param}")?;
                }
                write!(f, ") -> {ret}")
            }
            TypeKind::Hole(id) => write!(f, "_{id}"),
            TypeKind::Void => write!(f, "void"),
            TypeKind::Bool => write!(f, "bool"),
            TypeKind::Number(NumberKind::I8) => write!(f, "i8"),
            TypeKind::Number(NumberKind::I16) => write!(f, "i16"),
            TypeKind::Number(NumberKind::I32) => write!(f, "i32"),
            TypeKind::Number(NumberKind::I64) => write!(f, "i64"),
            TypeKind::Number(NumberKind::U8) => write!(f, "u8"),
            TypeKind::Number(NumberKind::U16) => write!(f, "u16"),
            TypeKind::Number(NumberKind::U32) => write!(f, "u32"),
            TypeKind::Number(NumberKind::U64) => write!(f, "u64"),
            TypeKind::Number(NumberKind::F32) => write!(f, "f32"),
            TypeKind::Number(NumberKind::F64 )=> write!(f, "f64"),
            TypeKind::Nothing => write!(f, "nothing"),
            TypeKind::Number(NumberKind::Number) => write!(f, "number"),
        }
    }
}

#[derive(Debug, Clone)]
struct GenericParam {
    name: Token,
    bounds: Vec<TypeKind>, // 泛型约束
}
