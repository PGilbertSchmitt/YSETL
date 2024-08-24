pub mod debug;

pub type Op = u8;

/** These specific values may change in the future */

// Value Operations
pub const CONST: Op = 1;
pub const NULL: Op = 2;
pub const TRUE: Op = 3;
pub const FALSE: Op = 4;
pub const SET_GLOBAL: Op = 5;
pub const GET_GLOBAL: Op = 6;
pub const SET_LOCAL: Op = 7;
pub const GET_LOCAL: Op = 8;

pub const MAKE_LIT_COL: Op = 9;
pub const MAKE_RN_COL: Op = 10;

// Stack Operations
pub const POP: Op = 20;

// Control Flow
pub const JUMP: Op = 30;
pub const JUMP_NOT_TRUE: Op = 31;
pub const JUMP_PEEK_AND: Op = 32;
pub const JUMP_PEEK_OR: Op = 33;
pub const JUMP_PEEK_NULL: Op = 34;

pub const RETURN: Op = 35;

// Builtins
pub const PRINT: Op = 150;

// Binary Operations
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
pub const GT: Op = 218;
pub const GTEQ: Op = 219;
pub const EQ: Op = 220;
pub const NEQ: Op = 221;
pub const LOGICAL_IMPL: Op = 222;

// Prefix Operations
pub const NOT: Op = 240;
pub const NEGATE: Op = 241;
pub const SIZE: Op = 242;
pub const HEAD: Op = 243;
pub const LAST: Op = 244;
pub const TAIL: Op = 245;
pub const INIT: Op = 246;

// DEBUG
pub const DBG_PRINT_STACK_TOP: Op = 40;
