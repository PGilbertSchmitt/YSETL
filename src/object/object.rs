use bytes::Bytes;
use once_cell::unsync::OnceCell;
use std::{
    collections::HashSet,
    hash::{DefaultHasher, Hash, Hasher},
    mem,
    rc::Rc,
};
use base64::{engine::general_purpose, Engine as _};
use rand::RngCore;

use crate::compiler::bytecode::{INCL_BIT, TUP_BASE};

#[derive(Debug, Clone)]
pub enum IterKind {
    Tuple,
    Set,
    String,
}

pub trait ObjectOps {
    fn is_truthy(&self) -> bool;
    fn is_null(&self) -> bool;
    fn truthy_convert(&self) -> Self;
    fn inner_int(&self) -> i64;
    fn inner_fn(&self) -> (Executor, usize, usize);

    fn make_range(range_start: i64, range_end: i64, step: Option<usize>, flag: u8) -> Self;

    fn to_s(&self) -> String;
    fn to_debug_string(&self) -> String;
}

pub trait FrameOps {
    fn to_vec(&self) -> Vec<Object>;
    fn iter_kind(&self) -> IterKind;
}

pub trait PreOps {
    fn not(self) -> Self;
    fn negate(self) -> Self;
    fn size(self) -> Self;
    fn head(self) -> Self;
    fn last(self) -> Self;
    fn tail(self) -> Self;
    fn init(self) -> Self;
}

#[derive(Debug, Clone)]
pub enum Object {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Atom(Atom),
    String {
        value: String,
        seed: Rc<OnceCell<u64>>,
    },
    Tuple {
        elements: Rc<Vec<Object>>,
        seed: Rc<OnceCell<u64>>,
    },
    Set {
        elements: Rc<HashSet<Object>>,
        seed: Rc<OnceCell<u64>>,
    },
    Closure {
        inner: Rc<Closure>,
        seed: Rc<OnceCell<u64>>,
    },
}

impl Object {
    pub fn new_string(s: String) -> Self {
        Self::String {
            value: s,
            seed: Rc::new(OnceCell::new()),
        }
    }

    pub fn new_tuple(elements: Vec<Object>) -> Self {
        Self::Tuple {
            elements: Rc::new(elements),
            seed: Rc::new(OnceCell::new()),
        }
    }

    pub fn new_set_from_vec(elements: Vec<Object>) -> Self {
        Self::Set {
            elements: Rc::new(HashSet::from_iter(elements)),
            seed: Rc::new(OnceCell::new()),
        }
    }

    pub fn new_set(elements: HashSet<Object>) -> Self {
        Self::Set {
            elements: Rc::new(elements),
            seed: Rc::new(OnceCell::new()),
        }
    }

    pub fn new_closure(executor: Executor, num_req_params: usize, num_opt_params: usize) -> Self {
        Self::Closure {
            inner: Rc::new(Closure {
                executor,
                num_req_params,
                num_opt_params,
            }),
            seed: Rc::new(OnceCell::new()),
            // function: Box::new(function),
        }
    }

    pub fn get_seed(&self) -> Option<&Rc<OnceCell<u64>>> {
        match self {
            Object::String { seed, .. } => Some(seed),
            Object::Tuple { seed, .. } => Some(seed),
            Object::Set { seed, .. } => Some(seed),
            Object::Closure { seed, .. } => Some(seed),
            _ => None,
        }
    }

    fn try_seed<F>(seed: &Rc<OnceCell<u64>>, generate_child_hash: F) -> u64
    where
        F: Fn(&mut DefaultHasher),
    {
        *seed
            .get_or_try_init(|| {
                let mut tmp_hasher = DefaultHasher::new();
                generate_child_hash(&mut tmp_hasher);
                Ok(tmp_hasher.finish()) as Result<u64, ()>
            })
            .unwrap()
    }
}

impl ObjectOps for Object {
    fn is_null(&self) -> bool {
        *self == Object::Null
    }

    fn is_truthy(&self) -> bool {
        match self {
            Self::Null | Self::Bool(false) | Self::Int(0) | Self::Float(0.0) => false,
            _ => true,
        }
    }

    fn truthy_convert(&self) -> Self {
        Self::Bool(self.is_truthy())
    }

    fn inner_int(&self) -> i64 {
        match &self {
            &Object::Int(x) => *x,
            _ => {
                panic!(
                    "Cannot convert value into integer: {}",
                    self.to_debug_string()
                );
            }
        }
    }

    fn inner_fn(&self) -> (Executor, usize, usize) {
        match &self {
            &Object::Closure { inner, .. } => {
                let exec = &inner.executor;
                (
                    Executor {
                        ins: exec.ins.clone(),
                        num_locals: exec.num_locals,
                        locked_values: exec.locked_values.clone(),
                    },
                    inner.num_req_params,
                    inner.num_opt_params,
                )
            }
            _ => panic!("Could not convert {self:?} into a function"),
        }
    }

    fn make_range(range_start: i64, range_end: i64, step: Option<usize>, flag: u8) -> Self {
        let inclusive = flag & INCL_BIT != 0;
        let elements: Vec<Object> = if range_start <= range_end {
            // Normal range
            let high = if inclusive { range_end + 1 } else { range_end };
            (range_start..high)
                .step_by(step.unwrap_or(1))
                .map(|i| Object::Int(i))
                .collect()
        } else {
            // Range starts from reverse
            let low = if inclusive { range_end } else { range_end + 1 };
            (low..range_start + 1)
                .rev()
                .step_by(step.unwrap_or(1))
                .map(|i| Object::Int(i))
                .collect()
        };

        if flag & TUP_BASE == 0 {
            Object::new_set_from_vec(elements)
        } else {
            Object::new_tuple(elements)
        }
    }

    fn to_s(&self) -> String {
        match self {
            Self::Null => String::from("null"),
            Self::Bool(val) => String::from(val.to_string()),
            Self::Int(x) => x.to_string(),
            Self::Float(x) => x.to_string(),
            Self::Atom(atom) => format!(":{}", atom.1),
            Self::String { value, .. } => value.clone(),
            Self::Tuple { elements, .. } => format!(
                "[{}]",
                elements
                    .iter()
                    .map(|o| o.to_s())
                    .collect::<Vec<String>>()
                    .join(",")
            ),
            Self::Set { elements, .. } => format!(
                "{{{}}}",
                elements
                    .iter()
                    .map(|o| o.to_s())
                    .collect::<Vec<String>>()
                    .join(",")
            ),
            // This could change if we also stored the function's string
            // along with the compliled data, but this is good enough for now
            Self::Closure { inner, .. } => format!(
                "fn({}, {}?) => [{} locals, {} bytes]",
                inner.num_req_params,
                inner.num_opt_params,
                inner.executor.num_locals,
                inner.executor.ins.len(),
            ),
        }
    }

    fn to_debug_string(&self) -> String {
        match self {
            Self::Int(x) => format!("i{x}"),
            Self::Float(x) => format!("f{x}"),
            Self::Atom(atom) => format!("sym({}):{}", atom.0, atom.1),
            Self::String { value, .. } => format!("\"{value}\""),
            Self::Tuple { elements, .. } => format!(
                "[{}]",
                elements
                    .iter()
                    .map(|o| o.to_debug_string())
                    .collect::<Vec<String>>()
                    .join(",")
            ),
            Self::Set { elements, .. } => format!(
                "{{{}}}",
                elements
                    .iter()
                    .map(|o| o.to_debug_string())
                    .collect::<Vec<String>>()
                    .join(",")
            ),
            _ => self.to_s(),
        }
    }
}

impl FrameOps for Object {
    fn to_vec(&self) -> Vec<Object> {
        match &self {
            &Object::Tuple { elements, .. } => elements.to_vec(),
            &Object::String { value, .. } => value
                .split("")
                .map(|str| Object::new_string(str.to_owned()))
                .collect(),
            _ => panic!("Cannot convert {} into list-like", self.to_debug_string()),
        }
    }

    fn iter_kind(&self) -> IterKind {
        match &self {
            &Object::Tuple { .. } => IterKind::Tuple,
            &Object::String { .. } => IterKind::String,
            _ => unimplemented!(),
        }
    }
}

impl PreOps for Object {
    fn not(self) -> Self {
        Self::Bool(!self.is_truthy())
    }

    fn negate(self) -> Self {
        match self {
            Object::Int(x) => Object::Int(-x),
            Object::Float(x) => Object::Float(-x),
            _ => {
                panic!("Cannot negate non-numeric value {}", self.to_debug_string())
            }
        }
    }
    
    fn size(self) -> Self {
        match self {
            Object::Null => panic!("Null has no cardinality"),
            Object::Bool(val) => if val { Object::Int(1) } else { Object::Int(0) }
            Object::Int(_) => self,
            Object::Float(v) => Object::Int(v.trunc() as i64),
            Object::Atom(a) => Object::Int(a.0 as i64),
            Object::String { value, .. } => Object::Int(value.len() as i64),
            Object::Tuple { elements, .. } => Object::Int(elements.len() as i64),
            Object::Set { elements, .. } => Object::Int(elements.len() as i64),
            Object::Closure { .. } => panic!("Cannot find cardinality of a function"),
        }
    }

    fn head(self) -> Self {
        match self {
            Object::Tuple { elements, .. } => {
                if let Some(first) = elements.first() {
                    first.clone()
                } else {
                    Object::Null
                }
            }
            Object::Set { elements, .. } => {
                if let Some(first) = elements.iter().next() {
                    first.clone()
                } else {
                    Object::Null
                }
            }
            Object::String { value, .. } => {
                if value.is_empty() {
                    Object::Null
                } else {
                    Object::new_string(value.get(0..1).unwrap().to_owned())
                }
            },
            _ => panic!("Cannot get the head of {}", self.to_s()),
        }
    }

    fn last(self) -> Self {
        match self {
            Object::Tuple { elements, .. } => {
                if let Some(last) = elements.last() {
                    last.clone()
                } else {
                    Object::Null
                }
            }
            // Set's don't have a particular order, but a single set value will always maintain
            // its iteration order. I don't have a better solution than just iterating, but these
            // aren't super useful operations for sets anyways. I may need to come up with
            // something better (and cooler).
            Object::Set { elements, .. } => {
                if let Some(first) = elements.iter().last() {
                    first.clone()
                } else {
                    Object::Null
                }
            }
            Object::String { value, .. } => {
                if value.is_empty() {
                    Object::Null
                } else {
                    let len = value.len();
                    Object::new_string(value.get(len-1..len).unwrap().to_owned())
                }
            },
            _ => panic!("Cannot get the last of {}", self.to_s()),
        }
    }

    fn tail(self) -> Self {
        todo!();
    }

    fn init(self) -> Self {
        todo!();
    }
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Object::Null, Object::Null) => true,
            (Object::Bool(x), Object::Bool(y)) => x == y,
            (Object::Int(x), Object::Int(y)) => x == y,
            (Object::Float(x), Object::Float(y)) => x == y,
            (Object::Atom(value), Object::Atom(other)) => value.0 == other.0,
            (Object::String { value, .. }, Object::String { value: other, .. }) => value == other,
            (
                Object::Tuple { elements, seed },
                Object::Tuple {
                    elements: other_elements,
                    seed: other_seed,
                    ..
                },
            ) => same_seed(seed, other_seed)
                .map_or_else(|| elements == other_elements, |is_same_seed| is_same_seed),
            (
                Object::Set { elements, seed },
                Object::Set {
                    elements: other_elements,
                    seed: other_seed,
                },
            ) => same_seed(seed, other_seed)
                .map_or_else(|| elements == other_elements, |is_same_seed| is_same_seed),
            (
                Object::Closure { inner, seed },
                Object::Closure {
                    inner: other_inner,
                    seed: other_seed,
                },
            ) => same_seed(seed, other_seed).map_or_else(
                || {
                    inner.executor == other_inner.executor
                        && inner.num_req_params == other_inner.num_req_params
                        && inner.num_opt_params == other_inner.num_opt_params
                },
                |is_same_seed| is_same_seed,
            ),
            _ => false,
        }
    }
}

impl Eq for Object {}

impl Hash for Object {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Object::Null => state.write_u8(b'n'),
            Object::Bool(val) => {
                state.write_u8(b'b');
                val.hash(state);
            }
            Object::Int(x) => {
                state.write_u8(b'i');
                x.hash(state);
            }
            Object::Float(x) => {
                state.write_u8(b'f');
                // All I need is that the bytes of the float make it into the hasher.
                // It doesn't really matter how "undefined" this behavior is.
                unsafe {
                    mem::transmute::<f64, u64>(*x).hash(state);
                }
            }
            Object::Atom(atom) => {
                state.write_u8(b'a');
                atom.0.hash(state);
            }
            Object::String { value, seed } => {
                state.write_u8(b's');
                let seed = Object::try_seed(seed, |h| value.hash(h));
                seed.hash(state);
            }
            Object::Tuple { elements, seed } => {
                state.write_u8(b't');
                let seed = Object::try_seed(seed, |h| elements.hash(h));
                seed.hash(state);
            }
            Object::Set { elements, seed } => {
                // This may be slower than XORing all element seeds together, but is much better for collisions
                // Maybe I didn't need to worry about this so much, and this might really only help if there
                // are a lot of sets being used in other sets or maps.
                state.write_u8(b'#'); // 's' was taken by String
                let seed = Object::try_seed(seed, |h| {
                    let mut element_subhashes = elements
                        .iter()
                        .map(|o| {
                            // This makes three sets of nested hashing, but 2 of those layers only need to be
                            // calculated once.
                            // I'm probably too dumb in Rust to figure out a better way to do this right now.
                            let mut sub_seed_hasher: DefaultHasher = DefaultHasher::new();
                            o.hash(&mut sub_seed_hasher);
                            sub_seed_hasher.finish()
                        })
                        .collect::<Vec<u64>>();
                    element_subhashes.sort();
                    element_subhashes.iter().for_each(|el| el.hash(h));
                });
                seed.hash(state);
            }
            Object::Closure { inner, seed } => {
                state.write_u8(b'c');
                let seed = Object::try_seed(seed, |h| {
                    inner.hash(h);
                });
                seed.hash(state);
            }
        }
    }
}

// This is separated from the Closure enum type for 3 reasons:
// - Re-used for Iterators
// - Will probably box this later to keep the BaseObject size small
// - Custom `PartialEq` implementation (just in case)
#[derive(Debug, Hash)]
pub struct Executor {
    pub ins: Bytes,
    pub num_locals: usize,
    pub locked_values: Rc<Vec<Object>>,
}

// TODO: Currently, equality between 2 functions checks that the instructions match
// (and that all the other inners match just in case). This could be pretty slow, so
// functions should be given unique IDs by the compiler.
// If function over-riding ever gets implemented, this equality check should also
// make sure that the overrides match.
impl PartialEq for Executor {
    fn eq(&self, other: &Self) -> bool {
        &self.ins == &other.ins && self.num_locals == self.num_locals
    }
}

#[derive(Debug, Hash)]
pub struct Closure {
    pub executor: Executor,
    num_req_params: usize,
    num_opt_params: usize,
}

fn same_seed(seed1: &Rc<OnceCell<u64>>, seed2: &Rc<OnceCell<u64>>) -> Option<bool> {
    if let (Some(self_hash), Some(other_hash)) = (
        once_cell::unsync::OnceCell::<u64>::get(seed1),
        once_cell::unsync::OnceCell::<u64>::get(seed2),
    ) {
        Some(self_hash == other_hash)
    } else {
        None
    }
}

#[derive(Clone, Debug)]
pub struct Atom (pub u32, String);

impl Atom {
    pub fn new(value: u32, name: String) -> Self {
        Self (value, name)
    }
    
    pub fn gen_atom_name() -> String {
        let mut data = [0u8; 9];
        rand::thread_rng().fill_bytes(&mut data);
        general_purpose::URL_SAFE.encode(&data)
    }
}
