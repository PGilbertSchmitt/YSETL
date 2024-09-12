use std::io::Cursor;

use bytes::{Buf, Bytes};

use crate::object::object::{Atom, Executor, Object};
use crate::op::lookup;

pub struct Bytecode {
    pub instructions: Bytes,
    pub constants: Vec<Object>,
    pub atoms: Vec<Atom>,
    pub iterators: Vec<Executor>,
    pub global_count: usize,
}

impl Bytecode {
    pub fn print_bytecode(&self) {
        print_bytecode(&self.instructions);
    }
}

pub fn print_bytecode(bytes: &Bytes) {
    let mut c = Cursor::new(bytes);

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

/* Collection creation flags
 *
 * - Collection Type: XX-- ----
 *   Reducer - 00
 *   Tuple   - 10
 *   Set     - 01
 *   Map     - 11
 *
 * - Inclusivity bit: --X- ----
 *   Exclusive - 0
 *   Inclusive - 1
 *
 * - Step bit: ---X ----
 *   Without Step - 0
 *   With Step    - 1
 *
 * - Reducer Op Bit: ---- X---
 *   With expression - 0
 *   With operator   - 1
 */

#[rustfmt::skip]
pub mod flags {
    pub const BASE_BITS:  u8 = 0b1100_0000;

    pub const RED_BASE:   u8 = 0b0000_0000; // Is that a TF2 reference!? No, short for reducer
    pub const TUP_BASE:   u8 = 0b1000_0000;
    pub const SET_BASE:   u8 = 0b0100_0000;
    pub const MAP_BASE:   u8 = 0b1100_0000;
    pub const INCL_BIT:   u8 = 0b0010_0000;
    pub const STEP_BIT:   u8 = 0b0001_0000;
    pub const RED_OP_BIT: u8 = 0b0000_1000;

    pub fn collection_flag_enabled(flags: u8, base: u8) -> bool {
        (flags & BASE_BITS) == base
    }
}
