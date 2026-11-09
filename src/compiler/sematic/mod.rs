use crate::compiler::Compiler;
use crate::compiler::com_error::SematicError;
use crate::compiler::ir::AstNode;
use crate::compiler::ir::hast::HastItem;
use crate::compiler::module::ModuleId;
use crate::compiler::sematic::resolution::resolution;
use crate::compiler::sematic::typeref::resolver;
use crate::compiler::symtbl::SymbolTable;

mod resolution;
mod typeref;

#[derive(Debug, Clone)]
pub enum SemanticLevel {
    Load,
    HighAst(Vec<HastItem>, Vec<SematicError>),
    Error,
}

pub struct Semantic<'a> {
    symtbl: SymbolTable<'a>,
    pub errors: Vec<SematicError>,
}

impl<'a> Semantic<'a> {
    #[must_use]
    pub fn new(symtbl: SymbolTable<'a>) -> Self {
        Self {
            symtbl,
            errors: Vec::new(),
        }
    }

    #[must_use]
    pub fn new_ir(compiler: &'a mut Compiler, id: ModuleId, errors: Vec<SematicError>) -> Self {
        Self {
            symtbl: SymbolTable::new(compiler, id),
            errors,
        }
    }

    pub fn get_mut_symtbl(&mut self) -> &mut SymbolTable<'a> {
        &mut self.symtbl
    }

    pub fn push_error(&mut self, err: SematicError) {
        self.errors.push(err);
    }

    pub fn resolver(&mut self, items: Vec<HastItem>) {
        resolver(self, items);
    }

    pub fn resolution(&mut self, nodes: Vec<AstNode>) -> Vec<HastItem> {
        resolution(self, nodes)
    }
}
