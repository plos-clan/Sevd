pub mod compiler;

use crate::compiler::Compiler;
use getopts_macro::getopts_options;
use mimalloc::MiMalloc;
use std::path::PathBuf;
use std::process::exit;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

struct Args {
    input: Vec<String>,
    name: Option<String>,
    version: bool,
}

impl Args {
    fn parser() -> Self {
        let options = getopts_options! {
            -v, --version       "Print version";
            -h, --help*         "Print help";
            -m, --modname*      "Set current module name";
        };

        let m = match options.parse(std::env::args().skip(1)) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("error: {e}");
                exit(2)
            }
        };
        if m.opt_present("help") {
            Self::help(&options);
            exit(1)
        }

        let args = Self {
            version: m.opt_present("version"),
            name: m.opt_strs("modname").iter().find_map(Self::parse_name),
            input: m.free,
        };
        args.check();
        args
    }

    fn check(&self) {
        if self.input.is_empty() && !self.version {
            eprintln!("error: required arguments were not provided: <INPUT>...");
            exit(2)
        }
    }

    fn help(options: &getopts_macro::getopts::Options) {
        let brief = format!(
            "Usage: {} [OPTIONS] [INPUT]...\n\n\
            Arguments:\n  [INPUT]...  the filename of the file to compile",
            Self::prog_name(),
        );
        print!("{}", options.usage(&brief));
    }

    fn parse_name(path: impl AsRef<str>) -> Option<String> {
        path.as_ref()
            .parse()
            .map_err(|e| eprintln!("warning: {e}"))
            .ok()
    }

    fn prog_name() -> String {
        std::env::args_os()
            .next()
            .and_then(|name| {
                PathBuf::from(name)
                    .file_name()?
                    .to_string_lossy()
                    .into_owned()
                    .into()
            })
            .unwrap_or_else(|| env!("CARGO_BIN_NAME").into())
    }
}

fn main() {
    let args = Args::parser();

    let mut compiler = Compiler::new(args.name.unwrap_or("default".to_string()));
    compiler.add_files(args.input);

    compiler.compile();
}
