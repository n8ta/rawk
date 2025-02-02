// #![feature(test)]
// extern crate test;

extern crate core;

// Jemalloc showed huge perf gains on malloc/free bound programs
// like evaluating a regex in a tight loop.
#[cfg(not(target_env = "msvc"))]
use jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

use crate::args::AwkArgs;
use crate::parser::Expr;
use crate::printable_error::PrintableError;

use crate::typing::AnalysisResults;

pub use crate::codegen::{compile_and_capture, compile_and_run};
pub use crate::lexer::lex;
pub use crate::parser::parse;
pub use crate::symbolizer::Symbolizer;
pub use crate::typing::analyze;

mod args;
mod codegen;
mod columns;
mod global_scalars;
mod integration_tests;
mod lexer;
mod parser;
mod printable_error;
mod runtime;
mod symbolizer;
mod typing;
mod awk_str;
mod util;

pub const PRINTF_MAX_ARGS: usize = 128;

pub fn runner(args: Vec<String>) -> Result<(), PrintableError> {
    let args = AwkArgs::new(args)?;

    let mut symbolizer = Symbolizer::new();
    let ast = analyze(parse(
        lex(&args.program, &mut symbolizer)?,
        &mut symbolizer,
    )?)?;
    if args.debug {
        println!("{}", ast);
    }

    if args.debug {
        codegen::compile_and_capture(ast, &args.files, &mut symbolizer, true)?;
    } else {
        codegen::compile_and_run(ast, &args.files, &mut symbolizer)?;
    }
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if let Err(err) = runner(args) {
        eprintln!("{}", err);
    }

    // Fuck cleanup just sys call out so it's faster
    unsafe { libc::exit(0 as libc::c_int) }
}
