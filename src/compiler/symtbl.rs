use crate::compiler::Compiler;
use crate::compiler::com_error::SematicError;
use crate::compiler::ir::hir::FieldId;
use crate::compiler::ir::tnode::TypeKind;
use crate::compiler::lexer::Token;
use crate::compiler::module::ModuleId;
use std::collections::HashMap;

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

#[derive(Debug, Clone)]
pub enum ScopeType {
    Loop,
    Func,
    Blk,
    Root,
}

#[derive(Debug, Clone)]
pub struct HoleTable {
    table: HashMap<usize, TypeKind>,
    index: usize,
}

impl HoleTable {
    pub fn new() -> Self {
        Self {
            table: HashMap::new(),
            index: 0,
        }
    }

    pub fn get_hole(&mut self, index: usize) -> Option<&TypeKind> {
        self.table.get(&index)
    }

    pub fn set_hole(&mut self, index: usize, ty: TypeKind) {
        self.table.insert(index, ty);
    }

    pub fn alloc_hole(&mut self) -> u64 {
        let index = self.index;
        self.index += 1;
        self.table.insert(index, TypeKind::Hole(index as u64));
        index as u64
    }
}

#[derive(Debug, Clone)]
pub struct Scope {
    bindings: Vec<(Token, TypeKind)>,
    table: HoleTable,
    sty: ScopeType,
}

impl Scope {
    pub fn new(sty: ScopeType) -> Self {
        Self {
            bindings: Vec::new(),
            table: HoleTable::new(),
            sty,
        }
    }

    pub fn get_type(&self) -> &ScopeType {
        &self.sty
    }

    pub fn get_mut_hole(&mut self) -> &mut HoleTable {
        &mut self.table
    }

    pub fn find_binding(&self, name: &str) -> Option<&(Token, TypeKind)> {
        self.bindings
            .iter()
            .find(|(token, _)| token.get_span().text() == name)
    }

    pub fn create_binding(&mut self, token: Token, sty: TypeKind) {
        self.bindings.push((token, sty));
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct SymbolTable<'a> {
    compiler: &'a mut Compiler,
    current_module: ModuleId,
    scopes: Vec<Scope>,
}

impl<'a> SymbolTable<'a> {
    pub fn new(compiler: &'a mut Compiler, current_module: ModuleId) -> Self {
        SymbolTable {
            scopes: vec![Scope::new(ScopeType::Root)],
            compiler,
            current_module,
        }
    }

    pub fn top_mut_scope(&mut self) -> &mut Scope {
        self.scopes.last_mut().unwrap()
    }

    pub fn push_scope(&mut self, ty: ScopeType) {
        self.scopes.push(Scope::new(ty));
    }

    pub fn exit_scope(&mut self) {
        self.scopes.pop();
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
