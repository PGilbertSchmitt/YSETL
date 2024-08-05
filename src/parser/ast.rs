#[derive(Debug)]
pub enum BinOp {
    Nullcoel,
    Take,
    Exp,
    Mult,
    Div,
    Mod,
    Add,
    Subtract,
    WithBitLeft, // With / Bitshift Left
    LessBitRight, // Less / Bitshift Right
    BitAnd,
    BitOr,
    BitXor,
    In,
    Notin,
    Subset,
    Lt,
    Lteq,
    Gt,
    Gteq,
    Eq,
    Neq,
    And,
    Or,
    Impl,
}

#[derive(Debug)]
pub enum PreOp {
    Not,
    Identity,
    Negate,
    Size,
    Head,
    Last,
    Tail,
    Init,
}

#[derive(Debug)]
pub enum Former {
    Empty,
    Literal(Vec<ExprST>),
    Range {
        inclusive: bool,
        start: Box<ExprST>,
        end: Box<ExprST>,
        step: Option<Box<ExprST>>,
    },
}

#[derive(Debug)]
pub enum ExprST {
    Null,
    Newat,
    True,
    False,
    Atom(String),
    String(String),
    Ident(String),
    Integer(i64),
    Float(f64),
    Tuple(Former),
    Set(Former),
    Infix {
        op: BinOp,
        lfs: Box<ExprST>,
        rhs: Box<ExprST>,
    },
    Prefix {
        op: PreOp,
        rhs: Box<ExprST>,
    }
}
