use std::io::Cursor;

use bytes::{Buf, Bytes};

use crate::{object::BaseObject, op::debug::lookup};

pub struct Bytecode {
    pub instructions: Bytes,
    pub constants: Vec<BaseObject>, 
    pub global_count: u16,
}

impl Bytecode {
    pub fn print_bytecode(&self) {
        let mut c = Cursor::new(&self.instructions);

        while c.has_remaining() {
            let pos = c.position();
            let op = c.get_u8();
            let (name, operands) = lookup(op);

            let operands_string = operands
                .iter()
                .map(|x| match *x {
                    1 => c.get_u8().to_string(),
                    2 => c.get_u16().to_string(),
                    4 => c.get_u32().to_string(),
                    _ => unimplemented!(),
                })
                .collect::<Vec<String>>()
                .join(" ");

            println!("{pos}: {name} {}", operands_string);
        }
    }
}

/**
 * Collection creation flags
 *
 * - Collection Type: XX-- ----
 *   Tuple - 10
 *   Set   - 01
 *
 * - Inclusivity bit: --X- ----
 *   Exclusive - 0
 *   Inclusive - 1
 *
 * - Step bit: ---X ----
 *   Without Step - 0
 *   With Step    - 1
 */

pub const TUP_BASE: u8  = 0b1000_0000;
pub const SET_BASE: u8  = 0b0100_0000;
pub const INCL_BIT: u8  = 0b0010_0000;
pub const STEP_BIT: u8 = 0b0001_0000;
