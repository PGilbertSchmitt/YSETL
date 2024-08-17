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
            let pos = c.position();
            let op = c.get_u8();
            let (name, operands) = lookup(op);

            match operands {
                [] => println!("{pos}: {name}"),
                [2] => println!("{pos}: {name} {}", c.get_u16()),
                [4] => println!("{pos}: {name} {}", c.get_u32()),
                _ => unreachable!(),
            }
        }
    }
}
