use crate::codegen::function_codegen::FunctionCodegen;
use crate::codegen::ValueT;
use crate::parser::{TypedExpr};
use crate::printable_error::PrintableError;
use crate::typing::BuiltinFunc;
use gnu_libjit::Value;

impl<'a> FunctionCodegen<'a> {
    fn arg_to_float(&mut self, args: &Vec<TypedExpr>, idx: usize) -> Result<Value, PrintableError> {
        let arg = self.compile_expr(&args[idx], false)?;
        let float = self.val_to_float(&arg, args[idx].typ);
        self.drop_if_str(arg, args[idx].typ);
        Ok(float)
    }
    fn arg_to_str(&mut self, args: &Vec<TypedExpr>, idx: usize) -> Result<Value, PrintableError> {
        let arg = self.compile_expr(&args[idx], false)?;
        let float = self.val_to_string(&arg, args[idx].typ);
        Ok(float)
    }
    fn mk_float(&self, flt: Value) -> ValueT {
        ValueT::new(self.float_tag(), flt, self.zero_ptr())
    }
    pub fn compile_builtin(
        &mut self,
        builtin: &BuiltinFunc,
        args: &Vec<TypedExpr>,
    ) -> Result<ValueT, PrintableError> {
        match builtin {
            BuiltinFunc::Sin => {
                let float = self.arg_to_float(args, 0)?;
                let sin = self.function.insn_sin(&float);
                Ok(self.mk_float(sin))
            }
            BuiltinFunc::Cos => {
                let float = self.arg_to_float(args, 0)?;
                let cos = self.function.insn_cos(&float);
                Ok(self.mk_float(cos))
            }
            BuiltinFunc::Log => {
                let float = self.arg_to_float(args, 0)?;
                let exp = self.function.insn_log(&float);
                Ok(self.mk_float(exp))
            }
            BuiltinFunc::Exp => {
                let float = self.arg_to_float(args, 0)?;
                let exp = self.function.insn_exp(&float);
                Ok(self.mk_float(exp))
            }
            BuiltinFunc::Sqrt => {
                let float = self.arg_to_float(args, 0)?;
                let sqrt = self.function.insn_sqrt(&float);
                Ok(self.mk_float(sqrt))
            }
            BuiltinFunc::Int => {
                let compiled = self.compile_expr(&args[0], false)?;
                let float = self.val_to_float(&compiled, args[0].typ);
                let floored = self.function.insn_trunc(&float);
                self.drop_if_str(compiled, args[0].typ);
                Ok(self.mk_float(floored))
            }
            BuiltinFunc::Rand => {
                let rnd = self.runtime.rand(&mut self.function);
                Ok(self.mk_float(rnd))
            }
            BuiltinFunc::Srand => {
                let float = self.arg_to_float(args, 0)?;
                let prior_seed = self.runtime.srand(&mut self.function, float);
                Ok(self.mk_float(prior_seed))
            }
            BuiltinFunc::Atan2 => {
                let f0 = self.arg_to_float(args, 0)?;
                let f1 = self.arg_to_float(args, 1)?;
                let atan2 = self.function.insn_atan2(&f0, &f1);
                Ok(self.mk_float(atan2))
            }
            BuiltinFunc::Length => {
                let str = self.arg_to_str(args, 0)?;
                let len = self.runtime.length(&mut self.function, str); //drops str
                Ok(self.mk_float(len))
            }
            BuiltinFunc::Split => todo!(),
            BuiltinFunc::Close => todo!(),
            BuiltinFunc::Gsub => todo!(),
            BuiltinFunc::Index => todo!(),
            BuiltinFunc::Matches => todo!(),
            BuiltinFunc::Sprintf => todo!(),
            BuiltinFunc::Sub => todo!(),
            BuiltinFunc::Substr => todo!(),
            BuiltinFunc::System => todo!(),
            BuiltinFunc::Tolower => {
                let compiled = self.compile_expr(&args[0], false)?;
                let string = self.val_to_string(&compiled, args[0].typ);
                let ptr = self.runtime.to_lower(&mut self.function, string);
                Ok(ValueT::new(self.string_tag(), self.zero_f(), ptr))
            }
            BuiltinFunc::Toupper => {
                let compiled = self.compile_expr(&args[0], false)?;
                let string = self.val_to_string(&compiled, args[0].typ);
                let ptr = self.runtime.to_upper(&mut self.function, string);
                Ok(ValueT::new(self.string_tag(), self.zero_f(), ptr))
            }
        }
    }
}