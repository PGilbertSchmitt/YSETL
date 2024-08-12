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
    Rest,
    RestOver(String),
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
    pub iterators: IteratorList,
    pub filters: ExprList,
}

#[derive(Debug)]
pub struct Range {
    pub inclusive: bool,
    pub start: Box<Expr>,
    pub end: Box<Expr>,
    pub step: Option<Box<Expr>>,
}

#[derive(Debug)]
pub enum Former {
    Empty,
    Literal(ExprList),
    Range(Range),
    Iterator {
        output: Box<Expr>,
        iterator: Iterator,
    },
}

#[derive(Debug)]
pub enum SelectOp {
    EXISTS,
    CHOOSE,
    FORALL,
}

#[derive(Debug)]
pub struct SwitchCase {
    pub condition: Option<Expr>,
    pub consequence: Expr,
}

#[derive(Debug)]
pub enum Postfix {
    Call(ExprList),
    Index(Box<Expr>),
    Slice {
        inclusive: bool,
        start: Option<Box<Expr>>,
        end: Option<Box<Expr>>,
    },
    Pick(ExprList),
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
    Postfix {
        lhs: Box<Expr>,
        postfix: Postfix,
    },
    Block {
        stmts: StmtList,
        implicit_return: Option<Box<Stmt>>,
    },
    Select {
        op: SelectOp,
        iterator: Iterator,
    },
    Ternary {
        condition: Box<Expr>,
        consequence: Box<Expr>,
        alternative: Box<Expr>,
    },
    Switch {
        condition: Option<Box<Expr>>,
        cases: CaseList,
    },
}

#[derive(Debug)]
pub enum Stmt {
    Expr(Expr),
    Return(Option<Expr>),
    Print(Expr),
    Assign { target: Bound, value: Expr },
}

pub type ExprList = Vec<Expr>;
pub type StmtList = Vec<Stmt>;
pub type BoundList = Vec<Bound>;
pub type IteratorList = Vec<SingleIterator>;
pub type CaseList = Vec<SwitchCase>;
