use clap::{arg, Parser, ValueEnum};
use execution::{
    interpreter::Interpreter,
    native::{codegen::CodeGeneration, state::State},
};
use optimize::{
    peephole::{CombineIncrements, ReplaceSet},
    OptimizationPass,
};

pub mod execution;
pub mod optimize;
pub mod syntax;
use std::{io::Write, path::PathBuf};

macro_rules! time {
    ($e:expr) => {{
        let start = std::time::Instant::now();
        let result = $e;
        let elapsed = start.elapsed();
        std::io::stdout().flush().unwrap();
        println!("Time: {:?}", elapsed);
        result
    }};
}

#[derive(clap::Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(value_enum, default_value_t=Mode::Compiled)]
    mode: Mode,

    #[arg(short, long)]
    path: PathBuf,

    #[arg(short, long)]
    optimize: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Mode {
    Compiled,
    Interpreted,
}

fn main() {
    let cli = Cli::parse();

    let s = std::fs::read_to_string(&cli.path).unwrap();
    let mut nodes = syntax::parse(&s).unwrap();

    if cli.optimize {
        nodes = CombineIncrements.optimize(nodes);
        nodes = ReplaceSet.optimize(nodes);
    }

    if true {
        let mut file = std::fs::File::create("optimized.txt").unwrap();
        writeln!(file, "{}", syntax::indented(&nodes, 0)).unwrap();
    }

    match cli.mode {
        Mode::Interpreted => {
            time!(Interpreter::new(30_000).interpret(&nodes));
        }
        Mode::Compiled => {
            let codegen = CodeGeneration::x64();
            let executor = codegen.generate(&nodes);
            let mut state = State::new(Box::new(std::io::stdin()), Box::new(std::io::stdout()));
            let result = time!(executor.run(&mut state));
            if result.is_error() {
                eprintln!("Error: {:?}", result);
            }
        }
    }
}
