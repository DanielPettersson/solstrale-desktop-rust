//! Arithmetic expressions for scene values, e.g. `sin(frameIndex * 0.1) * 2`.

use std::collections::BTreeSet;
use std::fmt;
use std::sync::Arc;

use crate::model::scope::Scope;

/// Functions an expression can call, with their allowed argument counts.
pub const FUNCTIONS: &[(&str, usize, usize)] = &[
    ("sin", 1, 1),
    ("cos", 1, 1),
    ("tan", 1, 1),
    ("abs", 1, 1),
    ("sqrt", 1, 1),
    ("floor", 1, 1),
    ("round", 1, 1),
    ("pow", 2, 2),
    ("min", 1, usize::MAX),
    ("max", 1, usize::MAX),
    ("len", 1, 3),
];

#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    /// Character offset into the source
    pub pos: usize,
    pub message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at position {}", self.message, self.pos + 1)
    }
}

impl std::error::Error for ParseError {}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Pow,
}

#[derive(Debug, Clone, PartialEq)]
enum Ast {
    Num(f64),
    Var(String),
    Neg(Box<Ast>),
    Bin(Op, Box<Ast>, Box<Ast>),
    Call(String, Vec<Ast>),
}

/// A parsed expression. Always holds source that parses.
#[derive(Clone)]
pub struct Expr {
    src: String,
    ast: Arc<Ast>,
}

impl Expr {
    pub fn parse(src: &str) -> Result<Expr, ParseError> {
        let tokens = tokenize(src)?;
        let mut parser = Parser {
            tokens,
            i: 0,
            end: src.chars().count(),
        };
        let ast = parser.expr()?;
        if let Some((pos, t)) = parser.tokens.get(parser.i) {
            return Err(ParseError {
                pos: *pos,
                message: format!("unexpected {}", t),
            });
        }
        Ok(Expr {
            src: src.to_string(),
            ast: Arc::new(ast),
        })
    }

    pub fn src(&self) -> &str {
        &self.src
    }

    pub fn eval(&self, scope: &Scope) -> Result<f64, String> {
        let v = eval(&self.ast, scope)?;
        if v.is_finite() {
            Ok(v)
        } else {
            Err(format!("result {} is not a finite number", v))
        }
    }

    /// Names of the variables the expression reads
    pub fn free_vars(&self) -> BTreeSet<String> {
        let mut vars = BTreeSet::new();
        collect_vars(&self.ast, &mut vars);
        vars
    }
}

impl PartialEq for Expr {
    fn eq(&self, other: &Self) -> bool {
        self.src == other.src
    }
}

impl fmt::Debug for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Expr({:?})", self.src)
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Num(f64),
    Ident(String),
    Sym(char),
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Num(n) => write!(f, "number {}", n),
            Token::Ident(s) => write!(f, "`{}`", s),
            Token::Sym(c) => write!(f, "`{}`", c),
        }
    }
}

fn tokenize(src: &str) -> Result<Vec<(usize, Token)>, ParseError> {
    let chars: Vec<char> = src.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() {
            i += 1;
        } else if c.is_ascii_digit() || c == '.' {
            let start = i;
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                i += 1;
            }
            if i < chars.len() && (chars[i] == 'e' || chars[i] == 'E') {
                let mut j = i + 1;
                if j < chars.len() && (chars[j] == '+' || chars[j] == '-') {
                    j += 1;
                }
                if j < chars.len() && chars[j].is_ascii_digit() {
                    i = j;
                    while i < chars.len() && chars[i].is_ascii_digit() {
                        i += 1;
                    }
                }
            }
            let text: String = chars[start..i].iter().collect();
            let n = text.parse::<f64>().map_err(|_| ParseError {
                pos: start,
                message: format!("invalid number `{}`", text),
            })?;
            tokens.push((start, Token::Num(n)));
        } else if c.is_alphabetic() || c == '_' {
            let start = i;
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            tokens.push((start, Token::Ident(chars[start..i].iter().collect())));
        } else if "+-*/%^(),".contains(c) {
            tokens.push((i, Token::Sym(c)));
            i += 1;
        } else {
            return Err(ParseError {
                pos: i,
                message: format!("unexpected character `{}`", c),
            });
        }
    }
    Ok(tokens)
}

struct Parser {
    tokens: Vec<(usize, Token)>,
    i: usize,
    end: usize,
}

impl Parser {
    fn peek_sym(&self) -> Option<char> {
        match self.tokens.get(self.i) {
            Some((_, Token::Sym(c))) => Some(*c),
            _ => None,
        }
    }

    fn expect(&mut self, c: char) -> Result<(), ParseError> {
        if self.peek_sym() == Some(c) {
            self.i += 1;
            Ok(())
        } else {
            Err(self.unexpected(&format!("expected `{}`", c)))
        }
    }

    fn unexpected(&self, expected: &str) -> ParseError {
        match self.tokens.get(self.i) {
            Some((pos, t)) => ParseError {
                pos: *pos,
                message: format!("{}, found {}", expected, t),
            },
            None => ParseError {
                pos: self.end,
                message: format!("{}, found end of expression", expected),
            },
        }
    }

    fn expr(&mut self) -> Result<Ast, ParseError> {
        let mut lhs = self.term()?;
        while let Some(c @ ('+' | '-')) = self.peek_sym() {
            self.i += 1;
            let rhs = self.term()?;
            let op = if c == '+' { Op::Add } else { Op::Sub };
            lhs = Ast::Bin(op, Box::new(lhs), Box::new(rhs));
        }
        Ok(lhs)
    }

    fn term(&mut self) -> Result<Ast, ParseError> {
        let mut lhs = self.unary()?;
        while let Some(c @ ('*' | '/' | '%')) = self.peek_sym() {
            self.i += 1;
            let rhs = self.unary()?;
            let op = match c {
                '*' => Op::Mul,
                '/' => Op::Div,
                _ => Op::Rem,
            };
            lhs = Ast::Bin(op, Box::new(lhs), Box::new(rhs));
        }
        Ok(lhs)
    }

    /// Unary minus binds looser than `^`, so `-2^2` is -4.
    fn unary(&mut self) -> Result<Ast, ParseError> {
        match self.peek_sym() {
            Some('-') => {
                self.i += 1;
                Ok(Ast::Neg(Box::new(self.unary()?)))
            }
            Some('+') => {
                self.i += 1;
                self.unary()
            }
            _ => self.power(),
        }
    }

    fn power(&mut self) -> Result<Ast, ParseError> {
        let base = self.primary()?;
        if self.peek_sym() == Some('^') {
            self.i += 1;
            let exp = self.unary()?;
            return Ok(Ast::Bin(Op::Pow, Box::new(base), Box::new(exp)));
        }
        Ok(base)
    }

    fn primary(&mut self) -> Result<Ast, ParseError> {
        let Some((pos, token)) = self.tokens.get(self.i).cloned() else {
            return Err(self.unexpected("expected a value"));
        };
        match token {
            Token::Num(n) => {
                self.i += 1;
                Ok(Ast::Num(n))
            }
            Token::Ident(name) => {
                self.i += 1;
                if self.peek_sym() != Some('(') {
                    return Ok(Ast::Var(name));
                }
                self.i += 1;
                let mut args = Vec::new();
                if self.peek_sym() != Some(')') {
                    args.push(self.expr()?);
                    while self.peek_sym() == Some(',') {
                        self.i += 1;
                        args.push(self.expr()?);
                    }
                }
                self.expect(')')?;
                check_call(&name, args.len(), pos)?;
                Ok(Ast::Call(name, args))
            }
            Token::Sym('(') => {
                self.i += 1;
                let inner = self.expr()?;
                self.expect(')')?;
                Ok(inner)
            }
            Token::Sym(_) => Err(self.unexpected("expected a value")),
        }
    }
}

fn check_call(name: &str, args: usize, pos: usize) -> Result<(), ParseError> {
    let Some((_, min, max)) = FUNCTIONS.iter().find(|(n, _, _)| *n == name) else {
        return Err(ParseError {
            pos,
            message: format!("unknown function `{}`", name),
        });
    };
    if args < *min || args > *max {
        let expected = if min == max {
            format!("{}", min)
        } else if *max == usize::MAX {
            format!("at least {}", min)
        } else {
            format!("{} to {}", min, max)
        };
        return Err(ParseError {
            pos,
            message: format!("`{}` takes {} arguments, got {}", name, expected, args),
        });
    }
    Ok(())
}

fn eval(ast: &Ast, scope: &Scope) -> Result<f64, String> {
    Ok(match ast {
        Ast::Num(n) => *n,
        Ast::Var(name) => scope
            .get(name)
            .ok_or_else(|| format!("unknown variable `{}`", name))?,
        Ast::Neg(a) => -eval(a, scope)?,
        Ast::Bin(op, a, b) => {
            let (a, b) = (eval(a, scope)?, eval(b, scope)?);
            match op {
                Op::Add => a + b,
                Op::Sub => a - b,
                Op::Mul => a * b,
                Op::Div => a / b,
                Op::Rem => a % b,
                Op::Pow => a.powf(b),
            }
        }
        Ast::Call(name, args) => {
            let args = args
                .iter()
                .map(|a| eval(a, scope))
                .collect::<Result<Vec<_>, _>>()?;
            match name.as_str() {
                "sin" => args[0].sin(),
                "cos" => args[0].cos(),
                "tan" => args[0].tan(),
                "abs" => args[0].abs(),
                "sqrt" if args[0] < 0. => {
                    return Err(format!("sqrt of negative number {}", args[0]));
                }
                "sqrt" => args[0].sqrt(),
                "floor" => args[0].floor(),
                "round" => args[0].round(),
                "pow" => args[0].powf(args[1]),
                "min" => args.iter().copied().fold(f64::INFINITY, f64::min),
                "max" => args.iter().copied().fold(f64::NEG_INFINITY, f64::max),
                "len" => args.iter().map(|a| a * a).sum::<f64>().sqrt(),
                _ => unreachable!("function names are checked when parsing"),
            }
        }
    })
}

fn collect_vars(ast: &Ast, vars: &mut BTreeSet<String>) {
    match ast {
        Ast::Num(_) => {}
        Ast::Var(name) => {
            vars.insert(name.clone());
        }
        Ast::Neg(a) => collect_vars(a, vars),
        Ast::Bin(_, a, b) => {
            collect_vars(a, vars);
            collect_vars(b, vars);
        }
        Ast::Call(_, args) => args.iter().for_each(|a| collect_vars(a, vars)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ev(src: &str) -> f64 {
        Expr::parse(src).unwrap().eval(&Scope::builtin(0)).unwrap()
    }

    #[test]
    fn precedence_and_associativity() {
        assert_eq!(7., ev("1 + 2 * 3"));
        assert_eq!(9., ev("(1 + 2) * 3"));
        assert_eq!(2., ev("8 / 2 / 2"));
        assert_eq!(512., ev("2 ^ 3 ^ 2"));
        assert_eq!(-4., ev("-2 ^ 2"));
        assert_eq!(0.5, ev("2 ^ -1"));
        assert_eq!(1., ev("7 % 3"));
        assert_eq!(0.5, ev("1 / 2"));
        assert_eq!(3., ev("--3"));
        assert_eq!(1500., ev("1.5e3"));
    }

    #[test]
    fn functions_and_constants() {
        assert!((ev("sin(pi / 2)") - 1.).abs() < 1e-12);
        assert!((ev("cos(0)") - 1.).abs() < 1e-12);
        assert!(ev("tan(0)").abs() < 1e-12);
        assert_eq!(3., ev("abs(-3)"));
        assert_eq!(3., ev("sqrt(9)"));
        assert_eq!(2., ev("floor(2.7)"));
        assert_eq!(3., ev("round(2.5)"));
        assert_eq!(8., ev("pow(2, 3)"));
        assert_eq!(1., ev("min(3, 1, 2)"));
        assert_eq!(3., ev("max(3, 1, 2)"));
        assert_eq!(5., ev("len(3, 4)"));
        assert_eq!(3., ev("len(1, 2, 2)"));
        assert!((ev("e") - std::f64::consts::E).abs() < 1e-12);
    }

    #[test]
    fn variables() {
        let scope = Scope::builtin(4).with("i", 2.);
        let e = Expr::parse("frameIndex * i + x_1").unwrap();
        assert_eq!(
            vec!["frameIndex", "i", "x_1"],
            e.free_vars().into_iter().collect::<Vec<_>>()
        );
        assert_eq!(Err("unknown variable `x_1`".to_string()), e.eval(&scope));
        assert_eq!(Ok(9.), e.eval(&scope.with("x_1", 1.)));
    }

    #[test]
    fn eval_errors() {
        let scope = Scope::builtin(0);
        assert!(Expr::parse("sqrt(-1)").unwrap().eval(&scope).is_err());
        assert!(Expr::parse("1 / 0").unwrap().eval(&scope).is_err());
    }

    #[test]
    fn parse_errors() {
        let err = |src: &str| Expr::parse(src).unwrap_err();
        assert_eq!(
            ParseError {
                pos: 4,
                message: "expected a value, found end of expression".to_string()
            },
            err("1 + ")
        );
        assert_eq!(0, err("foo(1)").pos);
        assert!(err("foo(1)").message.contains("unknown function `foo`"));
        assert!(err("pow(1)").message.contains("takes 2 arguments, got 1"));
        assert!(err("len(1, 2, 3, 4)").message.contains("1 to 3"));
        assert!(err("(1 + 2").message.contains("expected `)`"));
        assert!(err("1 2").message.contains("unexpected number 2"));
        assert_eq!(2, err("1 # 2").pos);
        assert!(err("").message.contains("expected a value"));
    }
}
