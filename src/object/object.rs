use base64::{engine::general_purpose, Engine as _};
use bytes::Bytes;
use nohash_hasher::{self, BuildNoHashHasher};
use once_cell::unsync::OnceCell;
use rand::RngCore;
use std::{
    hash::{BuildHasher, Hash, Hasher},
    mem,
    rc::Rc,
};
use xxhash_rust::xxh3::Xxh3;

use crate::{
    compiler::bytecode::flags::{INCL_BIT, TUP_BASE},
    vm::iterator::CollectionKind,
};

use super::hashing_collection::{new_y_set_from_vec, YsetlMap, YsetlSet};

#[derive(Debug, Clone)]
pub enum IterKind {
    Tuple,
    Set,
    String,
    Map,
}

pub trait ObjectOps {
    fn len(&self) -> usize;
    fn is_truthy(&self) -> bool;
    fn is_null(&self) -> bool;
    fn truthy_convert(&self) -> Self;
    fn inner_int(&self) -> i64;
    fn inner_fn(&self) -> (Executor, usize, usize);
    fn can_reduce(&self) -> bool;
    fn inner_tuple(&self) -> Vec<Object>;
    fn inner_set(&self) -> YsetlSet;
    fn inner_map(&self) -> YsetlMap;
    fn is_zero(&self) -> bool;
    fn ord_flt(&self) -> f64;
    fn insert(&self, key: Object, right: Object) -> Object;

    fn make_range(range_start: i64, range_end: i64, step: Option<usize>, flag: u8) -> Self;

    fn to_s(&self) -> String;
    fn to_debug_string(&self) -> String;
}

pub trait FrameOps {
    fn to_collection(&self) -> CollectionKind;
    // fn to_map_collection(&self) -> CollectionKind;
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
        elements: Rc<YsetlSet>,
        seed: Rc<OnceCell<u64>>,
    },
    Map {
        elements: Rc<YsetlMap>,
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
            elements: Rc::new(new_y_set_from_vec(elements)),
            seed: Rc::new(OnceCell::new()),
        }
    }

    pub fn new_set(elements: YsetlSet) -> Self {
        Self::Set {
            elements: Rc::new(elements),
            seed: Rc::new(OnceCell::new()),
        }
    }

    pub fn new_map(elements: YsetlMap) -> Self {
        Self::Map {
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

    fn to_key_string(&self) -> String {
        match self {
            Self::String { .. } => self.to_debug_string(),
            Self::Atom(atom) => atom.1.clone(),
            Self::Int(val) => format!("({val})"),
            Self::Float(val) => format!("({val})"),
            _ => self.to_s(),
        }
    }
}

impl ObjectOps for Object {
    fn len(&self) -> usize {
        match &self {
            &Object::String { value, .. } => value.len(),
            &Object::Tuple { elements, .. } => elements.len(),
            &Object::Set { elements, .. } => elements.len(),
            &Object::Map { elements, .. } => elements.len(),
            _ => panic!("Cannot calculate length of non-collection"),
        }
    }

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
            _ => panic!("Could not interpret {self:?} as a function"),
        }
    }

    // Hyper specific, but better here than in the VM
    fn can_reduce(&self) -> bool {
        match &self {
            &Object::Closure { inner, .. } => {
                let min = inner.num_req_params;
                let max = min + inner.num_opt_params;
                min <= 2 && 2 <= max
            }
            _ => panic!("Could not interpret {self:?} as a function"),
        }
    }

    fn inner_tuple(&self) -> Vec<Object> {
        match &self {
            &Object::Tuple { elements, .. } => (*elements.clone()).clone(),
            _ => panic!("Could not convert {self:?} into a vector"),
        }
    }

    fn inner_set(&self) -> YsetlSet {
        match &self {
            &Object::Set { elements, .. } => (*elements.clone()).clone(),
            _ => panic!("Could not convert {self:?} into a set"),
        }
    }

    fn inner_map(&self) -> YsetlMap {
        match &self {
            &Object::Map { elements, .. } => (*elements.clone()).clone(),
            _ => panic!("Could not convert {self:?} into a map"),
        }
    }

    fn is_zero(&self) -> bool {
        match self {
            Object::Int(0) => true,
            Object::Float(0.0) => true,
            _ => false,
        }
    }

    fn ord_flt(&self) -> f64 {
        match self {
            Object::Int(x) => *x as f64,
            Object::Float(x) => *x,
            Object::Set { elements, .. } => elements.len() as f64,
            _ => panic!("Cannot compare value {}", self.to_debug_string()),
        }
    }

    fn insert(&self, key: Object, value: Object) -> Object {
        match self {
            Object::Map { elements, .. } => {
                let mut new_elements = (*elements.clone()).clone();
                new_elements.insert(key, value);
                Object::new_map(new_elements)
            }
            _ => panic!("Cannot insert into value {}", self.to_s()),
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
                    .join(", ")
            ),
            Self::Set { elements, .. } => format!(
                "{{{}}}",
                elements
                    .iter()
                    .map(|o| o.to_s())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Self::Map { elements, .. } => format!(
                "{{{}}}",
                elements
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k.to_key_string(), v.to_s()))
                    .collect::<Vec<String>>()
                    .join(", ")
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
                    .join(", ")
            ),
            Self::Set { elements, .. } => format!(
                "s{{{}}}",
                elements
                    .iter()
                    .map(|o| o.to_debug_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Self::Map { elements, .. } => format!(
                "m{{{}}}",
                elements
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k.to_debug_string(), v.to_debug_string()))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            _ => self.to_s(),
        }
    }
}

impl FrameOps for Object {
    fn to_collection(&self) -> CollectionKind {
        match self {
            Object::String { value, .. } => CollectionKind::ListLike(
                value
                    .split("")
                    .map(|str| Object::new_string(str.to_owned()))
                    .collect(),
            ),
            Object::Tuple { elements, .. } => {
                CollectionKind::ListLike((*elements.clone()).clone().into_iter().collect())
            }
            Object::Set { elements, .. } => {
                CollectionKind::ListLike((*elements.clone()).clone().into_iter().collect())
            }
            Object::Map { elements, .. } => {
                CollectionKind::MapLike((*elements.clone()).clone().into_iter().collect())
            }
            _ => panic!("Cannot convert {} into a list-like collection", self.to_s()),
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
            Object::Bool(val) => {
                if val {
                    Object::Int(1)
                } else {
                    Object::Int(0)
                }
            }
            Object::Int(_) => self,
            Object::Float(v) => Object::Int(v.trunc() as i64),
            Object::Atom(a) => Object::Int(a.0 as i64),
            Object::String { value, .. } => Object::Int(value.len() as i64),
            Object::Tuple { elements, .. } => Object::Int(elements.len() as i64),
            Object::Set { elements, .. } => Object::Int(elements.len() as i64),
            Object::Map { elements, .. } => Object::Int(elements.len() as i64),
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
            }
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
                    Object::new_string(value.get(len - 1..len).unwrap().to_owned())
                }
            }
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
            ) => same_seed(seed, other_seed).unwrap_or_else(|| elements == other_elements),
            (
                Object::Set { elements, seed },
                Object::Set {
                    elements: other_elements,
                    seed: other_seed,
                },
            ) => same_seed(seed, other_seed).unwrap_or_else(|| elements == other_elements),
            (
                Object::Closure { inner, seed },
                Object::Closure {
                    inner: other_inner,
                    seed: other_seed,
                },
            ) => same_seed(seed, other_seed).unwrap_or_else(|| {
                inner.executor == other_inner.executor
                    && inner.num_req_params == other_inner.num_req_params
                    && inner.num_opt_params == other_inner.num_opt_params
            }),
            (
                Object::Map { elements, seed },
                Object::Map {
                    elements: other_elements,
                    seed: other_seed,
                },
            ) => same_seed(seed, other_seed).unwrap_or_else(|| elements == other_elements),
            _ => false,
        }
    }
}

impl Eq for Object {}

impl nohash_hasher::IsEnabled for Object {}

impl Hash for Object {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Object::Null => state.write_u8(b'n'),
            Object::Bool(val) => val.hash(state),
            Object::Int(x) => state.write_i64(*x),
            // All I need is that the bytes of the float make it into the hasher.
            // It doesn't really matter how "undefined" this behavior is.
            Object::Float(x) => unsafe {
                state.write_u64(mem::transmute::<f64, u64>(*x));
            },
            Object::Atom(atom) => state.write_u32(atom.0),
            Object::String { value, seed } => {
                seed.get_or_init(|| {
                    let mut xh = Xxh3::new();
                    xh.update(value.as_bytes());
                    xh.write_u8(b's');
                    xh.digest()
                })
                .hash(state);
            }
            Object::Tuple { elements, seed } => {
                seed.get_or_init(|| {
                    let mut xh = Xxh3::new();
                    elements.hash(&mut xh);
                    xh.write_u8(b't');
                    xh.digest()
                })
                .hash(state);
            }
            Object::Set { elements, seed } => {
                seed.get_or_init(|| {
                    let mut element_subhashes: Vec<u64> = elements
                        .iter()
                        .map(|o| {
                            let mut no_hasher: nohash_hasher::NoHashHasher<u64> =
                                BuildNoHashHasher::default().build_hasher();
                            o.hash(&mut no_hasher);
                            no_hasher.finish()
                        })
                        .collect();
                    element_subhashes.sort();
                    let mut xh = Xxh3::new();
                    element_subhashes
                        .iter()
                        .for_each(|subhash| xh.write_u64(*subhash));
                    xh.write_u8(b'#');
                    xh.finish()
                })
                .hash(state);
            }
            Object::Map { elements, seed } => {
                seed.get_or_init(|| {
                    let mut element_subhashes: Vec<u64> = elements
                        .iter()
                        .map(|o| {
                            let mut sub_xh = Xxh3::new();
                            o.hash(&mut sub_xh);
                            sub_xh.finish()
                        })
                        .collect();
                    element_subhashes.sort();
                    let mut xh = Xxh3::new();
                    element_subhashes
                        .iter()
                        .for_each(|subhash| xh.write_u64(*subhash));
                    xh.write_u8(b'm');
                    xh.finish()
                })
                .hash(state);
            }
            Object::Closure { inner, seed } => {
                seed.get_or_init(|| {
                    let mut xh = Xxh3::new();
                    inner.hash(&mut xh);
                    xh.write_u8(b'c');
                    xh.finish()
                })
                .hash(state);
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
pub struct Atom(pub u32, String);

impl Atom {
    pub fn new(value: u32, name: String) -> Self {
        Self(value, name)
    }

    pub fn gen_atom_name() -> String {
        let mut data = [0u8; 9];
        rand::thread_rng().fill_bytes(&mut data);
        general_purpose::URL_SAFE.encode(&data)
    }
}
