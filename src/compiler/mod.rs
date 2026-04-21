pub mod com_error;
pub mod ir;
pub mod lexer;
pub mod module;
pub mod parser;
pub mod sematic;
mod symtbl;
mod typedef;

use std::{
    fs,
    path::{Component, Path},
};

use crate::compiler::com_error::print_parser_error;
use crate::compiler::lexer::LexerAnalysis;
use crate::compiler::module::{Module, SubModule};
use crate::compiler::parser::Parser;
use crate::compiler::symtbl::SymbolTable;
use line_column::span::Span;

#[derive(Debug, Clone)]
pub struct Compiler {
    mods: Vec<Module>,
    compiling: Module,
}

impl Compiler {
    pub fn new(name: String) -> Self {
        Self {
            mods: vec![],
            compiling: Module::new(name),
        }
    }

    pub fn normalize_path(path_str: &str) -> Vec<String> {
        Path::new(path_str)
            .components()
            .filter_map(|component| match component {
                Component::CurDir => None,
                Component::Normal(segment) => Some(segment.to_string_lossy().into_owned()),
                Component::ParentDir => Some("..".to_string()),
                Component::RootDir | Component::Prefix(_) => None,
            })
            .collect()
    }

    fn read_source_file(path: &str, normalized_path: &[String]) -> SourceFile {
        let data =
            fs::read_to_string(path).unwrap_or_else(|e| panic!("error: cannot read file{e}"));
        SourceFile {
            data: Span::new_full(data),
            name: normalized_path.join("/"),
            symbol: false,
        }
    }

    fn ensure_submodule<'a>(
        mods: &'a mut Vec<SubModule>,
        directories: &[String],
    ) -> &'a mut SubModule {
        let (current, rest) = directories.split_first().unwrap();
        let index = if let Some(index) = mods.iter().position(|submodule| {
            matches!(
                submodule,
                SubModule::SubModule { name, .. } if name == current
            )
        }) {
            index
        } else {
            mods.push(SubModule::SubModule {
                name: current.clone(),
                root: None,
                mods: Vec::new(),
            });
            mods.len() - 1
        };

        let submodule = &mut mods[index];
        if rest.is_empty() {
            return submodule;
        }

        match submodule {
            SubModule::SubModule { mods, .. } => Self::ensure_submodule(mods, rest),
            SubModule::File(_) => unreachable!(),
        }
    }

    fn compile_submodules(compiler: &Compiler, mods: &mut [SubModule]) {
        for submodule in mods {
            match submodule {
                SubModule::File(file) => Self::compile_source_file(compiler, file),
                SubModule::SubModule { root, mods, .. } => {
                    if let Some(root) = root {
                        Self::compile_source_file(compiler, root);
                    }
                    Self::compile_submodules(compiler, mods);
                }
            }
        }
    }

    fn compile_source_file(compiler: &Compiler, file: &mut SourceFile) {
        println!("Compiling {}", file.name);
        file.compiler_header(compiler);
    }

    pub fn add_file(&mut self, path: &str) {
        self.add_files(vec![String::from(path)]);
    }

    pub fn add_files(&mut self, paths: Vec<String>) {
        for path in paths {
            let normalized_path = Self::normalize_path(path.as_str());
            if normalized_path.is_empty() {
                continue;
            }

            let file = Self::read_source_file(path.as_str(), &normalized_path);
            if normalized_path.len() == 1 {
                self.compiling.files.push(SubModule::File(file));
                continue;
            }

            let directories = &normalized_path[..normalized_path.len() - 1];
            let file_name = normalized_path.last().unwrap().as_str();
            let submodule = Self::ensure_submodule(&mut self.compiling.files, directories);
            match submodule {
                SubModule::SubModule { root, mods, .. } => {
                    if file_name == "mod.sev" {
                        *root = Some(file);
                    } else {
                        mods.push(SubModule::File(file));
                    }
                }
                SubModule::File(_) => unreachable!(),
            }
        }
    }

    pub fn compile(&mut self) {
        let mut files = std::mem::take(&mut self.compiling.files);
        Self::compile_submodules(self, &mut files);
        self.compiling.files = files;
    }
}

#[derive(Debug, Clone)]
pub struct SourceFile {
    pub name: String,
    pub data: Span,
    pub symbol: bool, // 是否为仅符号识别解析 (不会输出目标文件)
}

impl SourceFile {
    pub fn compiler_header(&mut self, compiler: &Compiler) {
        let mut lex = LexerAnalysis::new(self);
        let mut parser = Parser::new(&mut lex);
        let mut symtbl = SymbolTable::new(compiler);
        match parser.parser(&mut symtbl) {
            Ok(nodes) => {
                dbg!(nodes);
            }
            Err(err) => print_parser_error(self, err),
        }
    }
}
