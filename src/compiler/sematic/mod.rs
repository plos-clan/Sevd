use crate::compiler::SourceFile;
use crate::compiler::com_error::{SematicError, print_semantic_error};
use crate::compiler::ir::AstNode;
use crate::compiler::sematic::resolution::resolution;
use crate::compiler::symtbl::SymbolTable;

pub mod resolution;

pub struct Semantic<'a> {
    file: &'a SourceFile,
    #[allow(dead_code)]
    symtbl: SymbolTable<'a>,
    errors: Vec<SematicError>,
}

impl<'a> Semantic<'a> {
    pub fn new(file: &'a SourceFile, symtbl: SymbolTable<'a>) -> Self {
        Self {
            file,
            symtbl,
            errors: Vec::new(),
        }
    }

    pub fn push_error(&mut self, err: SematicError) {
        self.errors.push(err);
    }

    pub fn semantic(&mut self, nodes: Vec<AstNode>) {
        let mast = resolution(self, nodes);
        if !self.errors.is_empty() {
            print_semantic_error(self.file, self.errors.to_vec());
            return;
        }

        dbg!(mast);
    }
}
