use action;
use env;
use list;
use lexer::Token;

pub struct ParseError {}
pub type ParseResult<T> = Result<T, ParseError>;

pub trait Parse {
    type Out;
    fn parse(&self) -> Self::Out;
}

pub struct GenericBinaryOp<Op, L, R, O> where L: Parse<Out=O>, R: Parse<Out=O> {
    op: Op,
    left: Box<L>,
    right: Box<R>,
}

pub enum BooleanOp {
    And,
    Or,
}

pub type BooleanExpr<L,R> = GenericBinaryOp<BooleanOp, L, R, bool>;

impl<L,R> Parse for BooleanExpr<L, R> where L: Parse<Out=bool>, R: Parse<Out=bool> {
    type Out = bool;
    fn parse(&self) -> bool {
        match self.op {
            BooleanOp::And => if !self.left.parse() { false } else { self.right.parse() },
            BooleanOp::Or => if self.left.parse() { true } else { self.right.parse() }
        }
    }
}

pub enum CompareOp {
    Equals,
    NotEquals,
    GreaterThan,
    GreaterThanEquals,
    LessThan,
    LessThanEquals
}

pub type CompareExpr<L,R,T> where T: PartialOrd = GenericBinaryOp<CompareOp, L, R, T>;

impl<L,R,T> Parse for CompareExpr<L,R,T> where L: Parse<Out=T>, R: Parse<Out=T>, T: PartialOrd {
    type Out = bool;
    fn parse(&self) -> bool {
        match self.op {
            CompareOp::Equals => self.left.parse().eq(&self.right.parse()),
            CompareOp::NotEquals => self.left.parse().ne(&self.right.parse()),
            CompareOp::GreaterThan => self.left.parse().gt(&self.right.parse()),
            CompareOp::GreaterThanEquals => self.left.parse().ge(&self.right.parse()),
            CompareOp::LessThan => self.left.parse().lt(&self.right.parse()),
            CompareOp::LessThanEquals => self.left.parse().le(&self.right.parse())
        }
    }
}


pub struct AssignmentExpr {
    pub local: bool,
    pub name: String,
    pub value: Option<String>
}

macro_rules! declaration {
    (
        $name:ident => { $($k:ident: $v:ty,)* }
    ) => {
        pub struct $name {
            name: String,
            $( $k: $v,)*
        }
    }
}

declaration!{ ActionDeclaration => {
    flags: Vec<action::Flags>,
    bind_list: Vec<String>,
    command: String,
}}

declaration!{ RuleDeclaration => {
    args: Vec<String>,
}}