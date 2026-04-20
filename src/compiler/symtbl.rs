use std::collections::HashMap;

use crate::compiler::Compiler;
use crate::compiler::typedef::TypeKind;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Element<'a> {
    pub type_kind: TypeKind<'a>,
    pub name: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum ContextType {
    Loop,
    Lambda,
    Function,
    Root,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Context<'a> {
    elements: Vec<Element<'a>>,
    types: ContextType,
}

impl<'a> Context<'a> {
    pub fn new(types: ContextType) -> Context<'a> {
        Self {
            elements: Vec::new(),
            types,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SymbolTable<'a> {
    context_stack: Vec<Context<'a>>,
    types: HashMap<String, TypeKind<'a>>,
    compiler: &'a Compiler,
}

impl<'a> SymbolTable<'a> {
    pub fn new(compiler: &'a Compiler) -> Self {
        let context_stack = vec![Context::new(ContextType::Root)];
        SymbolTable {
            context_stack,
            types: HashMap::new(),
            compiler,
        }
    }

    pub fn find_type(&self, name: &str) -> Option<TypeKind<'a>> {
        self.types.get(name).cloned()
    }

    pub fn add_type(&mut self, type_kind: TypeKind<'a>, name: String) {
        self.types.insert(name, type_kind);
    }

    pub fn add_element(&mut self, type_kind: TypeKind<'a>, name: String) {
        self.context_stack
            .last_mut()
            .unwrap()
            .elements
            .push(Element { name, type_kind })
    }

    pub fn compiler(&self) -> &'a Compiler {
        self.compiler
    }
}
