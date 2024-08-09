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
    WithBitLeft,  // With / Bitshift Left
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
pub enum Bound {
    Tilde,
    Ident(String),
    List(BoundList),
}

#[derive(Debug)]
pub enum SingleIterator {
    In {
        bounds: BoundList,
        expr: Expr,
    },
    SelectOne {
        bound: Bound,
        collection: String,
        list: BoundList,
    },
    SelectMany {
        bound: Bound,
        collection: String,
        list: BoundList,
    },
}

#[derive(Debug)]
pub struct Iterator {
    pub iterators: Vec<SingleIterator>,
    pub filters: ExprList,
}

#[derive(Debug)]
pub enum Former {
    Empty,
    Literal(ExprList),
    Range {
        inclusive: bool,
        start: Box<Expr>,
        end: Box<Expr>,
        step: Option<Box<Expr>>,
    },
    Iterator {
        output: Box<Expr>,
        iterator: Iterator,
    },
}

#[derive(Debug)]
pub enum Expr {
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
    Function {
        req_params: Vec<String>,
        opt_params: Vec<String>,
        eval: Box<Expr>,
    },
    Infix {
        op: BinOp,
        lfs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Prefix {
        op: PreOp,
        rhs: Box<Expr>,
    },
    Block {
        stmts: Vec<Stmt>,
        implicit_return: Option<Box<Stmt>>,
    },
}

#[derive(Debug)]
pub enum Stmt {
    Expr(Expr),
    Return(Option<Expr>),
    Print(Expr),
}

pub type ExprList = Vec<Expr>;
pub type StmtList = Vec<Stmt>;
pub type BoundList = Vec<Bound>;
pub type IteratorList = Vec<SingleIterator>;
