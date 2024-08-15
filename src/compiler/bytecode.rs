use std::io::Cursor;

use bytes::{Buf, Bytes};

use crate::{object::BaseObject, op::debug::lookup};

pub struct Bytecode {
    pub instructions: Bytes,
    pub constants: Vec<BaseObject>,
}

impl Bytecode {
    pub fn print_bytecode(&self) {
        let mut c = Cursor::new(&self.instructions);

        while c.has_remaining() {
            let op = c.get_u8();
            let (name, operands) = lookup(op);

            match operands {
                [] => println!("{name}"),
                [2] => println!("{name} {}", c.get_u16()),
                _ => unreachable!()
            }
        }
    }
}