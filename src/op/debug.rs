// Provides the name of an opcode, and the number and byte-lengths of its operands
pub fn lookup(byte: u8) -> (&'static str, &'static [u8]) {
    match byte {
        super::CONST => ("CONST", &[2]),
        super::NULL => ("NULL", &[]),
        super::TRUE => ("TRUE", &[]),
        super::FALSE => ("FALSE", &[]),

        super::POP => ("POP", &[]),
        super::PRINT => ("PRINT", &[]),

        super::NULLCOEL => ("NULLCOEL", &[]),
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
        super::EQ => ("EQ", &[]),
        super::NEQ => ("NEQ", &[]),

        super::LOGICAL_AND => ("LOGICAL_AND", &[]),
        super::LOGICAL_OR => ("LOGICAL_OR", &[]),
        super::LOGICAL_IMPL => ("LOGICAL_IMPL", &[]),

        _ => unreachable!(),
    }
}
