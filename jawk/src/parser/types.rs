use crate::lexer::{BinOp, LogicalOp, MathOp};
use std::fmt::{Display, Formatter};
use libc::write;
use crate::parser;

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ScalarType {
    String,
    Float,
    Variable,
}

#[derive(Debug, PartialEq)]
pub enum Stmt {
    Expr(TypedExpr),
    Print(TypedExpr),
    Group(Vec<Stmt>),
    If(TypedExpr, Box<Stmt>, Option<Box<Stmt>>),
    While(TypedExpr, Box<Stmt>),
    Printf { fstring: TypedExpr, args: Vec<TypedExpr> },
    Break,
}

impl Display for Stmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Stmt::Printf { fstring, args } => {
                write!(f, "printf \"{}\"", fstring)?;
                for mem in args {
                    write!(f, "{},", mem.expr)?;
                }
            }
            Stmt::Expr(expr) => write!(f, "{}", expr)?,
            Stmt::Print(expr) => write!(f, "print {}", expr)?,
            Stmt::Group(group) => {
                for elem in group {
                    write!(f, "{}", elem)?;
                }
            }
            Stmt::If(test, if_so, if_not) => {
                write!(f, "if {} {{{}}}", test, if_so)?;
                if let Some(else_case) = if_not {
                    write!(f, "else {{ {} }}", else_case)?;
                }
            }
            Stmt::While(test, body) => {
                write!(f, "while {} {{{}}} ", test, body)?;
            }
            Stmt::Break => write!(f, "break")?,
        };
        write!(f, "\n")
    }
}

#[derive(Debug, PartialEq)]
pub struct PatternAction {
    pub pattern: Option<TypedExpr>,
    pub action: Stmt,
}

impl PatternAction {
    pub fn new<ExprT: Into<Option<TypedExpr>>>(pattern: ExprT, action: Stmt) -> Self {
        Self { pattern: pattern.into(), action }
    }
    pub fn new_pattern_only(test: TypedExpr) -> PatternAction {
        PatternAction::new(
            Some(test),
            Stmt::Print(Expr::Column(Box::new(
                Expr::NumberF64(0.0).into()),
            ).into()),
        )
    }
    pub fn new_action_only(body: Stmt) -> PatternAction {
        PatternAction::new(None, body)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct TypedExpr {
    pub typ: ScalarType,
    pub expr: Expr,
}

impl TypedExpr {
    pub fn new(expr: Expr) -> TypedExpr {
        TypedExpr {
            typ: ScalarType::Variable,
            expr,
        }
    }
}

impl Into<TypedExpr> for Expr {
    fn into(self) -> TypedExpr {
        TypedExpr::new(self)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    ScalarAssign(String, Box<TypedExpr>),
    ArrayAssign { name: String, indices: Vec<TypedExpr>, value: Box<TypedExpr> },
    NumberF64(f64),
    String(String),
    Concatenation(Vec<TypedExpr>),
    BinOp(Box<TypedExpr>, BinOp, Box<TypedExpr>),
    MathOp(Box<TypedExpr>, MathOp, Box<TypedExpr>),
    LogicalOp(Box<TypedExpr>, LogicalOp, Box<TypedExpr>),
    Variable(String),
    Column(Box<TypedExpr>),
    Call,
    Ternary(Box<TypedExpr>, Box<TypedExpr>, Box<TypedExpr>),
    Regex(String),
    ArrayIndex { name: String, indices: Vec<TypedExpr> },
    InArray { name: String, indices: Vec<TypedExpr> },
}

impl Display for TypedExpr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.typ {
            ScalarType::String => write!(f, "(s {})", self.expr),
            ScalarType::Float => write!(f, "(f {})", self.expr),
            ScalarType::Variable => write!(f, "(v {})", self.expr),
        }
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::ScalarAssign(var, expr) => write!(f, "{} = {}", var, expr),
            Expr::Call => write!(f, "check_if_there_is_another_line"),
            Expr::Variable(n) => write!(f, "{}", n),
            Expr::String(str) => write!(f, "\"{}\"", str),
            Expr::NumberF64(n) => write!(f, "{}", n),
            Expr::BinOp(left, op, right) => write!(f, "{}{}{}", left, op, right),
            Expr::Ternary(cond, expr1, expr2) => write!(f, "{} ? {} : {}", cond, expr1, expr2),
            Expr::MathOp(left, op, right) => write!(f, "{}{}{}", left, op, right),
            Expr::LogicalOp(left, op, right) => write!(f, "{}{}{}", left, op, right),
            Expr::Column(col) => write!(f, "${}", col),
            Expr::Concatenation(vals) => {
                let vals = vals
                    .iter()
                    .map(|v| format!("{}", v))
                    .collect::<Vec<String>>();
                let str = vals.join(" ");
                write!(f, "{}", str)
            }
            Expr::Regex(str) => write!(f, "\"{}\"", str),

            Expr::ArrayIndex { name, indices } => {
                write!(f, "{}[", name)?;
                for idx in indices {
                    write!(f, "{},", idx)?;
                }
                write!(f, "]")
            }
            Expr::InArray { name, indices } => {
                write!(f, "(")?;
                for idx in indices {
                    write!(f, "{},", idx)?;
                }
                write!(f, ") in {}", name)
            }
            Expr::ArrayAssign { name, indices, value } => {
                write!(f, "{}[", name)?;
                for idx in indices {
                    write!(f, "{},", idx)?;
                }
                write!(f, "] = {}", value)
            }
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ArgT {
    Unused,
    Scalar,
    Array,
}

impl Display for ArgT {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ArgT::Unused => write!(f, "u"),
            ArgT::Scalar => write!(f, "s"),
            ArgT::Array => write!(f, "a"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Arg {
    pub name: String,
    pub typ: ArgT,
}

impl Display for Arg {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} {})", self.typ, self.name)
    }
}

#[derive(Debug, PartialEq)]
pub struct Function {
    pub name: String,
    pub args: Vec<Arg>,
    pub body: Stmt,
}

impl Function {
    pub fn new(name: String, args: Vec<String>, body: Stmt) -> Self {
        Function {
            name,
            args: args.into_iter().map(|arg| Arg { name: arg, typ: ArgT::Unused }).collect(),
            body,
        }
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "function {}(", self.name)?;
        for (idx, arg) in self.args.iter().enumerate() {
            write!(f, "{}", arg)?;
            if idx != self.args.len() - 1 {
                write!(f, ", ")?;
            }
        }
        write!(f, ") {{\n{}\n}}", self.body)
    }
}