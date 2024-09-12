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
pub const GET_LOCKED: Op = 9;

pub const MAKE_LIT_COL: Op = 10;
pub const MAKE_RN_COL: Op = 11;

pub const GET_ATOM: Op = 12;
pub const MAKE_ATOM: Op = 13;

pub const MAKE_FN: Op = 15;

// Stack Operations
pub const POP: Op = 20;
pub const PUSH_MATCH: Op = 21;
pub const POP_MATCH: Op = 22;

// Control Flow
pub const JUMP: Op = 30;
pub const JUMP_IF_FALSE: Op = 31;
pub const JUMP_IF_TRUE: Op = 32; // More convenient than pushing an extra NOT operation
pub const JUMP_PEEK_AND: Op = 33;
pub const JUMP_PEEK_OR: Op = 34;
pub const JUMP_PEEK_NULL: Op = 35;
pub const JUMP_NOT_MATCH: Op = 36;

pub const CALL: Op = 40;
pub const RETURN: Op = 41;

// Iterator-specific
pub const ITER_START: Op = 60;
pub const ITER_NEXT: Op = 61;
pub const ITER_COLLECT: Op = 62;
pub const ITER_END: Op = 63;
pub const GET_ITER_VAL: Op = 64;
pub const GET_ITER_KEY: Op = 65;
pub const ITER_EMPTY_CHECK: Op = 66;
pub const GET_ACC: Op = 67;
pub const REDUCE_CALL: Op = 68;
pub const REDUCE_WITH: Op = 69;
pub const MAKE_ITER: Op = 70;

// Builtins
pub const PRINT: Op = 150;
pub const PRINT_DBG: Op = 151;

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
pub const INSERT: Op = 223;

// Prefix Operations
pub const NOT: Op = 240;
pub const NEGATE: Op = 241;
pub const SIZE: Op = 242;
pub const HEAD: Op = 243;
pub const LAST: Op = 244;
pub const TAIL: Op = 245;
pub const INIT: Op = 246;

// DEBUG
pub const DBG_PRINT_STACK_TOP: Op = 255;

// Provides the name of an opcode, and the number and byte-lengths of its operands
pub const fn lookup(byte: u8) -> (&'static str, &'static [u8]) {
    match byte {
        CONST => ("CONST", &[2]),
        NULL => ("NULL", &[]),
        TRUE => ("TRUE", &[]),
        FALSE => ("FALSE", &[]),
        SET_GLOBAL => ("SET_GLOBAL", &[2]),
        GET_GLOBAL => ("GET_GLOBAL", &[2]),
        SET_LOCAL => ("SET_LOCAL", &[2]),
        GET_LOCAL => ("GET_LOCAL", &[2]),
        GET_LOCKED => ("GET_LOCKED", &[2]),
        MAKE_LIT_COL => ("MAKE_LIT_COL", &[1, 2]),
        MAKE_RN_COL => ("MAKE_RN_COL", &[1]),
        GET_ATOM => ("GET_ATOM", &[4]),
        MAKE_ATOM => ("MAKE_ATOM", &[]),
        MAKE_FN => ("MAKE_FN", &[2, 2]),

        POP => ("POP", &[]),
        PUSH_MATCH => ("PUSH_MATCH", &[]),
        POP_MATCH => ("POP_MATCH", &[]),

        JUMP => ("JUMP", &[4]),
        JUMP_IF_FALSE => ("JUMP_IF_FALSE", &[4]),
        JUMP_IF_TRUE => ("JUMP_IF_TRUE", &[4]),
        JUMP_PEEK_AND => ("JUMP_PEEK_AND", &[4]),
        JUMP_PEEK_OR => ("JUMP_PEEK_OR", &[4]),
        JUMP_PEEK_NULL => ("JUMP_PEEK_NULL", &[4]),
        JUMP_NOT_MATCH => ("JUMP_NOT_MATCH", &[4]),

        CALL => ("CALL", &[2]),
        RETURN => ("RETURN", &[]),

        ITER_START => ("ITER_START", &[2, 2, 1, 1]),
        ITER_NEXT => ("ITER_NEXT", &[1, 4]),
        ITER_COLLECT => ("ITER_COLLECT", &[]),
        ITER_END => ("ITER_END", &[]),
        MAKE_ITER => ("MAKE_ITER", &[1]),
        GET_ITER_VAL => ("GET_ITER_VAL", &[1]),
        GET_ITER_KEY => ("GET_ITER_KEY", &[1]),
        ITER_EMPTY_CHECK => ("ITER_EMPTY_CHECK", &[4]),
        GET_ACC => ("GET_ACC", &[]),
        REDUCE_CALL => ("REDUCE_CALL", &[]),
        REDUCE_WITH => ("REDUCE_WITH", &[1]),

        PRINT => ("PRINT", &[]),
        PRINT_DBG => ("PRINT_DBG", &[]),

        TAKE => ("TAKE", &[]),
        EXP => ("EXP", &[]),
        MULT => ("MULT", &[]),
        DIV => ("DIV", &[]),
        MOD => ("MOD", &[]),
        ADD => ("ADD", &[]),
        SUBTRACT => ("SUBTRACT", &[]),
        WITH_BIT_LEFT => ("WITH_BIT_LEFT", &[]),
        LESS_BIT_RIGHT => ("LESS_BIT_RIGHT", &[]),
        BIT_AND => ("BIT_AND", &[]),
        BIT_OR => ("BIT_OR", &[]),
        BIT_XOR => ("BIT_XOR", &[]),
        IN => ("IN", &[]),
        NOTIN => ("NOTIN", &[]),
        SUBSET => ("SUBSET", &[]),
        LT => ("LT", &[]),
        LTEQ => ("LTEQ", &[]),
        GT => ("GT", &[]),
        GTEQ => ("GTEQ", &[]),
        EQ => ("EQ", &[]),
        NEQ => ("NEQ", &[]),
        LOGICAL_IMPL => ("LOGICAL_IMPL", &[]),

        INSERT => ("INSERT", &[]),

        NOT => ("NOT", &[]),
        NEGATE => ("NEGATE", &[]),
        SIZE => ("SIZE", &[]),
        HEAD => ("HEAD", &[]),
        LAST => ("LAST", &[]),
        TAIL => ("TAIL", &[]),
        INIT => ("INIT", &[]),

        DBG_PRINT_STACK_TOP => ("DBG_PRINT_STACK_TOP", &[]),

        _ => panic!("No debug lookup for byte"),
    }
}
