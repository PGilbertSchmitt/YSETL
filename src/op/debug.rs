// Provides the name of an opcode, and the number and byte-lengths of its operands
pub fn lookup(byte: u8) -> (&'static str, &'static [u8]) {
    match byte {
        super::CONST => ("CONST", &[2]),
        super::NULL => ("NULL", &[]),
        super::TRUE => ("TRUE", &[]),
        super::FALSE => ("FALSE", &[]),
        super::ADD => ("ADD", &[]),
        _ => unimplemented!(),
    }
}
