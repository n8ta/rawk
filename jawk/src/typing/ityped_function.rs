use crate::parser::{Arg, ArgT};
use crate::symbolizer::Symbol;
use crate::typing::structs::Call;
use crate::typing::TypedUserFunction;
use crate::{AnalysisResults, PrintableError};
use hashbrown::HashSet;
use std::cell::Ref;
use std::fmt::{Debug, Display};
use std::rc::Rc;

pub trait ITypedFunction: Debug + Display {
    fn args(&self) -> Ref<'_, Vec<Arg>>;
    fn min_arity(&self) -> usize;
    fn add_caller(&self, caller: Rc<TypedUserFunction>);
    fn calls(&self) -> Ref<'_, Vec<Call>>;
    fn callers(&self) -> Ref<'_, HashSet<Rc<TypedUserFunction>>>;
    fn name(&self) -> Symbol;
    fn get_call_types(&self, program: &AnalysisResults, link: &Call) -> Vec<ArgT>;

    // We are function A
    // A --> B
    // B has new information about the link from A to B
    fn reverse_call(
        &self,
        link: &Call,
        args: &Vec<Arg>,
        analysis: &mut AnalysisResults,
    ) -> Result<HashSet<Symbol>, PrintableError>;

    // We are function B
    // A --> B
    // A has new information about the link from A to B
    fn receive_call(&self, call: &Vec<ArgT>) -> Result<HashSet<Symbol>, PrintableError>;
}
impl PartialEq for dyn ITypedFunction {
    fn eq(&self, other: &Self) -> bool {
        self.name() == other.name()
    }
}
