pub mod com_error;
pub mod ir;
pub mod lexer;
pub mod module;
pub mod parser;
pub mod sematic;
mod symtbl;

use crate::compiler::com_error::print_parser_error;
use crate::compiler::lexer::LexerAnalysis;
use crate::compiler::module::{Module, ModuleId};
use crate::compiler::parser::Parser;
use crate::compiler::sematic::Semantic;
use crate::compiler::symtbl::SymbolTable;
use line_column::span::Span;
use std::{
    fs, io,
    path::{Component, Path},
};

#[derive(Debug, Clone)]
pub struct Compiler {
    alloc_id: usize,
    mods: Vec<Module>,
    compiling: Module,
}

impl Compiler {
    pub fn new(name: String) -> Self {
        Self {
            alloc_id: 1,
            mods: vec![],
            compiling: Module::new(ModuleId(0), name, None),
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

    fn read_source_file(path: &str, normalized_path: &[String]) -> io::Result<SourceFile> {
        let data = fs::read_to_string(path)?;
        Ok(SourceFile {
            data: Span::new_full(data),
            name: normalized_path.join("/"),
            symbol: false,
        })
    }

    fn ensure_submodule<'a>(
        mut module: &'a mut Module,
        parts: &[String],
        alloc_id: &mut usize,
    ) -> &'a mut Module {
        for name in parts {
            let index = if let Some(index) = module.modules.iter().position(|m| m.name == *name) {
                index
            } else {
                let id = ModuleId(*alloc_id);
                *alloc_id += 1;
                module.modules.push(Module::new(id, name.clone(), None));
                module.modules.len() - 1
            };
            module = &mut module.modules[index];
        }
        module
    }

    fn compile_submodules(compiler: &Compiler, module: &Module, path: &mut Vec<String>) {
        if let Some(file) = &module.file {
            println!("Compiling {} - {}", file.name, path.join("."));
            file.compiler_header(compiler, module.id);
        }
        for child in &module.modules {
            path.push(child.name.clone());
            Self::compile_submodules(compiler, child, path);
            path.pop();
        }
    }

    pub fn add_file(&mut self, path: &str) -> io::Result<()> {
        self.add_files(vec![String::from(path)])
    }

    pub fn add_files(&mut self, paths: Vec<String>) -> io::Result<()> {
        for path in paths {
            let normalized_path = Self::source_path(&path)?;
            let file = Self::read_source_file(&path, &normalized_path)?;
            self.add_source_file(&normalized_path, file)?;
        }
        Ok(())
    }

    // Source paths are relative to the working directory, our current source root.
    fn source_path(path: &str) -> io::Result<Vec<String>> {
        let cwd = std::env::current_dir()?;
        let input = Path::new(path);
        let relative = if input.is_absolute() {
            input.strip_prefix(&cwd).map_err(|_| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "source file is outside the source root",
                )
            })?
        } else {
            input
        };
        let mut parts = Vec::new();
        for component in relative.components() {
            match component {
                Component::CurDir => {}
                Component::Normal(name) => parts.push(
                    name.to_str()
                        .ok_or_else(|| {
                            io::Error::new(
                                io::ErrorKind::InvalidInput,
                                "source path is not valid UTF-8",
                            )
                        })?
                        .to_owned(),
                ),
                Component::ParentDir if !parts.is_empty() => {
                    parts.pop();
                }
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "source file is outside the source root",
                    ));
                }
            }
        }
        Ok(parts)
    }

    fn add_source_file(&mut self, source_path: &[String], file: SourceFile) -> io::Result<()> {
        let Some((filename, directories)) = source_path.split_last() else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "source path is empty",
            ));
        };
        if Path::new(filename).extension().and_then(|ext| ext.to_str()) != Some("sev") {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "source file must have a .sev extension",
            ));
        }
        let mut parts = directories.to_vec();
        if filename != "mod.sev" {
            let name = file
                .module_name()
                .filter(|name| !name.is_empty())
                .ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "source file has no module name",
                    )
                })?;
            parts.push(name.to_owned());
        }
        let module = Self::ensure_submodule(&mut self.compiling, &parts, &mut self.alloc_id);
        if let Some(existing) = &module.file {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!(
                    "module `{}` already has source `{}`; cannot add `{}`",
                    module.name, existing.name, file.name
                ),
            ));
        }
        module.file = Some(file);
        Ok(())
    }

    /// Resolve an absolute module path; `this` is an alias for the compiling root.
    pub fn resolve_module(&self, path: &[String]) -> Option<&Module> {
        let (name, rest) = path.split_first()?;
        let root = if name == "this" || *name == self.compiling.name {
            &self.compiling
        } else {
            self.mods.iter().find(|module| module.name == *name)?
        };
        root.find(rest)
    }

    pub fn module(&self, id: ModuleId) -> Option<&Module> {
        self.compiling
            .find_by_id(id)
            .or_else(|| self.mods.iter().find_map(|module| module.find_by_id(id)))
    }

    pub fn get_module_path(&self, id: ModuleId) -> Option<Vec<String>> {
        self.compiling.get_module_path_by_id(id).or_else(|| {
            self.mods
                .iter()
                .find_map(|module| module.get_module_path_by_id(id))
        })
    }

    pub fn compile(&self) {
        let mut path = vec![self.compiling.name.clone()];
        Self::compile_submodules(self, &self.compiling, &mut path);
    }
}

#[derive(Debug, Clone)]
pub struct SourceFile {
    pub name: String,
    pub data: Span,
    pub symbol: bool, // 是否为仅符号识别解析 (不会输出目标文件)
}

impl SourceFile {
    pub fn module_name(&self) -> Option<&str> {
        Path::new(&self.name).file_stem()?.to_str()
    }

    pub fn compiler_header(&self, compiler: &Compiler, module_id: ModuleId) {
        let mut lex = LexerAnalysis::new(self);
        let mut parser = Parser::new(&mut lex);
        let mut symtbl = SymbolTable::new(compiler, module_id);

        let nodes = match parser.parser(&mut symtbl) {
            Ok(nodes) => nodes,
            Err(err) => {
                print_parser_error(self, err);
                return;
            }
        };

        let mut semantic = Semantic::new(self, symtbl);
        semantic.semantic(nodes);
    }
}
