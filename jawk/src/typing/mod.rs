mod inference_pass;
mod function_pass;
mod types;
mod test;

pub use crate::typing::types::{TypedProgram, AnalysisResults, TypedFunc};

use crate::parser::{Program};
use crate::printable_error::PrintableError;
use crate::typing::function_pass::FunctionAnalysis;
use crate::typing::inference_pass::variable_inference;

pub fn analyze(stmt: Program) -> Result<Program, PrintableError> {
    let func_analysis = FunctionAnalysis::new();
    let typed_program = variable_inference(func_analysis.analyze_program(stmt)?)?;
    let prog = typed_program.done();
    Ok(prog)
}