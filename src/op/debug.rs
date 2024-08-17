// Provides the name of an opcode, and the number and byte-lengths of its operands
pub fn lookup(byte: u8) -> (&'static str, &'static [u8]) {
    match byte {
        super::CONST => ("CONST", &[2]),
        super::NULL => ("NULL", &[]),
        super::TRUE => ("TRUE", &[]),
        super::FALSE => ("FALSE", &[]),

        super::POP => ("POP", &[]),
        super::PRINT => ("PRINT", &[]),

        super::JUMP => ("JUMP", &[4]),
        super::JUMP_NOT_TRUE => ("JUMP_NOT_TRUE", &[4]),
        super::JUMP_PEEK_AND => ("JUMP_PEEK_AND", &[4]),
        super::JUMP_PEEK_OR => ("JUMP_PEEK_OR", &[4]),
        super::JUMP_PEEK_NULL => ("JUMP_PEEK_NULL", &[4]),

        super::RETURN => ("RETURN", &[]),

        super::TAKE => ("TAKE", &[]),
        super::EXP => ("EXP", &[]),
        super::MULT => ("MULT", &[]),
        super::DIV => ("DIV", &[]),
        super::MOD => ("MOD", &[]),
        super::ADD => ("ADD", &[]),
        super::SUBTRACT => ("SUBTRACT", &[]),
        super::WITH_BIT_LEFT => ("WITH_BIT_LEFT", &[]),
        super::LESS_BIT_RIGHT => ("LESS_BIT_RIGHT", &[]),
        super::BIT_AND => ("BIT_AND", &[]),
        super::BIT_OR => ("BIT_OR", &[]),
        super::BIT_XOR => ("BIT_XOR", &[]),
        super::IN => ("IN", &[]),
        super::NOTIN => ("NOTIN", &[]),
        super::SUBSET => ("SUBSET", &[]),
        super::LT => ("LT", &[]),
        super::LTEQ => ("LTEQ", &[]),
        super::GT => ("GT", &[]),
        super::GTEQ => ("GTEQ", &[]),
        super::EQ => ("EQ", &[]),
        super::NEQ => ("NEQ", &[]),
        super::LOGICAL_IMPL => ("LOGICAL_IMPL", &[]),

        super::NOT => ("NOT", &[]),
        super::NEGATE => ("NEGATE", &[]),
        super::SIZE => ("SIZE", &[]),
        super::HEAD => ("HEAD", &[]),
        super::LAST => ("LAST", &[]),
        super::TAIL => ("TAIL", &[]),
        super::INIT => ("INIT", &[]),

        super::DBG_PRINT_STACK_TOP => ("DBG_PRINT_STACK_TOP", &[]),

        _ => panic!("No debug lookup for byte: {}", byte),
    }
}
