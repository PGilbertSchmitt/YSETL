pub mod debug;

pub type Op = u8;

/** These specific values may change in the future */

// Value Operations
pub const CONST: Op = 1;
pub const NULL: Op = 2;
pub const TRUE: Op = 3;
pub const FALSE: Op = 4;

// Stack Operations
pub const POP: Op = 20;

// Builtins
pub const PRINT: Op = 150;

// Binary Operations
pub const NULLCOEL: Op = 200;
pub const TAKE: Op = 201;
pub const EXP: Op = 202;
pub const MULT: Op = 203;
pub const DIV: Op = 204;
pub const MOD: Op = 205;
pub const ADD: Op = 206;
pub const SUBTRACT: Op = 207;
pub const WITH_BIT_LEFT: Op = 208;
pub const LESS_BIT_RIGHT: Op = 209;
pub const BIT_AND: Op = 210;
pub const BIT_OR: Op = 211;
pub const BIT_XOR: Op = 212;
pub const IN: Op = 213;
pub const NOTIN: Op = 214;
pub const SUBSET: Op = 215;
pub const LT: Op = 216;
pub const LTEQ: Op = 217;
pub const EQ: Op = 218;
pub const NEQ: Op = 219;

// Prefix Operations
pub const NOT: Op = 240;
pub const NEGATE: Op = 241;
pub const SIZE: Op = 242;
pub const HEAD: Op = 243;
pub const LAST: Op = 244;
pub const TAIL: Op = 245;
pub const INIT: Op = 246;

// Binary Branch Operations
pub const LOGICAL_AND: Op = 250;
pub const LOGICAL_OR: Op = 251;
pub const LOGICAL_IMPL: Op = 252;
