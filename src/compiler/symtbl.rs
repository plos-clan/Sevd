use crate::compiler::Compiler;
use crate::compiler::com_error::SematicError;
use crate::compiler::ir::hir::FieldId;
use crate::compiler::lexer::Token;
use crate::compiler::module::ModuleId;

#[derive(Debug, Clone)]
pub enum ItemType {
    Enum,
    Struct,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Item {
    alloc_field_id: usize,
    pub name: Token,
    fields: Vec<(FieldId, Token)>,
    types: ItemType,
}

impl Item {
    #[must_use]
    pub fn new(name: Token, types: ItemType) -> Self {
        Self {
            alloc_field_id: 0,
            name,
            fields: vec![],
            types,
        }
    }

    fn alloc_field_id(&mut self) -> FieldId {
        let id = FieldId(self.alloc_field_id);
        self.alloc_field_id += 1;
        id
    }

    pub fn add_field(&mut self, name: Token) -> Result<FieldId, SematicError> {
        if self
            .fields
            .iter()
            .any(|(_, existing)| existing.get_span().text() == name.get_span().text())
        {
            return Err(SematicError::FiledAlreadyDefined(name));
        }
        let id = self.alloc_field_id();
        self.fields.push((id, name));
        Ok(id)
    }
}

#[derive(Debug)]
pub struct SymbolTable<'a> {
    compiler: &'a mut Compiler,
    current_module: ModuleId,
}

impl<'a> SymbolTable<'a> {
    pub fn new(compiler: &'a mut Compiler, current_module: ModuleId) -> Self {
        SymbolTable {
            compiler,
            current_module,
        }
    }

    pub fn compiler(&self) -> &Compiler {
        self.compiler
    }

    pub fn compiler_mut(&mut self) -> &mut Compiler {
        self.compiler
    }

    pub fn current_module(&self) -> ModuleId {
        self.current_module
    }
}
