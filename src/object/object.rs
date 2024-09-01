use bytes::Bytes;
use once_cell::unsync::OnceCell;
use std::{
    collections::HashSet,
    hash::{DefaultHasher, Hash, Hasher},
    mem,
    rc::Rc,
};

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
    fn not(&self) -> Self;
    fn negate(&self) -> Self;
    fn inner_int(&self) -> i64;
    fn inner_fn(&self) -> (Executor, usize, usize);
    fn to_vec(&self) -> Vec<Object>;
    fn iter_kind(&self) -> IterKind;

    fn to_s(&self) -> String;
    fn to_debug_string(&self) -> String;
}

#[derive(Debug)]
pub enum BaseObject {
    Null,
    True,
    False,
    Int(i64),
    Float(f64),
    String(String),
    Tuple(Vec<Object>),
    Set(HashSet<Object>),
    Closure {
        function: Box<Executor>,
        num_req_params: usize,
        num_opt_params: usize,
    },
}

impl BaseObject {
    pub fn wrap(self) -> Object {
        Object {
            base: Rc::new(self),
            seed: Rc::new(OnceCell::new()),
        }
    }
}

impl ObjectOps for BaseObject {
    fn is_null(&self) -> bool {
        *self == BaseObject::Null
    }

    fn is_truthy(&self) -> bool {
        match self {
            Self::Null | Self::False | Self::Int(0) | Self::Float(0.0) => false,
            _ => true,
        }
    }

    fn truthy_convert(&self) -> Self {
        if self.is_truthy() {
            Self::True
        } else {
            Self::False
        }
    }

    fn not(&self) -> Self {
        if self.is_truthy() {
            Self::False
        } else {
            Self::True
        }
    }

    fn negate(&self) -> Self {
        match self {
            BaseObject::Int(x) => BaseObject::Int(-x),
            BaseObject::Float(x) => BaseObject::Float(-x),
            _ => {
                panic!("Cannot negate non-boolean value {}", self.to_debug_string())
            }
        }
    }

    fn inner_int(&self) -> i64 {
        match &self {
            &BaseObject::Int(x) => *x,
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
            &BaseObject::Closure {
                function,
                num_req_params,
                num_opt_params,
            } => (
                Executor {
                    ins: function.ins.clone(),
                    num_locals: function.num_locals,
                    locked_values: function.locked_values.clone(),
                },
                *num_req_params,
                *num_opt_params,
            ),
            _ => panic!("Could not convert {self:?} into a function"),
        }
    }

    fn to_vec(&self) -> Vec<Object> {
        match &self {
            &BaseObject::Tuple(vec) => vec.clone(),
            &BaseObject::String(str) => str
                .split("")
                .map(|str| BaseObject::String(str.to_owned()).wrap())
                .collect(),
            _ => panic!("Cannot convert {} into list-like", self.to_debug_string()),
        }
    }

    fn iter_kind(&self) -> IterKind {
        match &self {
            &BaseObject::Tuple(_) => IterKind::Tuple,
            &BaseObject::String(_) => IterKind::String,
            _ => unimplemented!(),
        }
    }

    fn to_s(&self) -> String {
        match self {
            Self::Null => String::from("null"),
            Self::False => String::from("false"),
            Self::True => String::from("true"),
            Self::Int(x) => x.to_string(),
            Self::Float(x) => x.to_string(),
            Self::String(val) => val.clone(),
            Self::Tuple(vals) => format!(
                "[{}]",
                vals.iter()
                    .map(|o| o.to_s())
                    .collect::<Vec<String>>()
                    .join(",")
            ),
            Self::Set(set) => format!(
                "{{{}}}",
                set.iter()
                    .map(|o| o.to_s())
                    .collect::<Vec<String>>()
                    .join(",")
            ),
            // This could change if we also stored the function's string
            // along with the compliled data, but this is good enough for now
            Self::Closure {
                function,
                num_req_params,
                num_opt_params,
            } => format!(
                "fn({}, {}?) => [{} locals, {} bytes]",
                num_req_params,
                num_opt_params,
                function.num_locals,
                function.ins.len(),
            ),
        }
    }

    fn to_debug_string(&self) -> String {
        match self {
            Self::Int(x) => format!("i{x}"),
            Self::Float(x) => format!("f{x}"),
            Self::String(val) => format!("\"{val}\""),
            Self::Tuple(vals) => format!(
                "[{}]",
                vals.iter()
                    .map(|o| o.to_debug_string())
                    .collect::<Vec<String>>()
                    .join(",")
            ),
            Self::Set(set) => format!(
                "{{{}}}",
                set.iter()
                    .map(|o| o.to_debug_string())
                    .collect::<Vec<String>>()
                    .join(",")
            ),
            _ => self.to_s(),
        }
    }
}

impl PartialEq for BaseObject {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (BaseObject::Null, BaseObject::Null) => true,
            (BaseObject::True, BaseObject::True) => true,
            (BaseObject::False, BaseObject::False) => true,
            (BaseObject::Int(x), BaseObject::Int(y)) => x == y,
            (BaseObject::String(x), BaseObject::String(y)) => x == y,
            (BaseObject::Tuple(x), BaseObject::Tuple(y)) => x == y,
            (
                BaseObject::Closure {
                    function,
                    num_opt_params,
                    num_req_params,
                },
                BaseObject::Closure {
                    function: other_function,
                    num_opt_params: other_num_opt_params,
                    num_req_params: other_num_req_params,
                },
            ) => {
                function == other_function
                    && num_req_params == other_num_req_params
                    && num_opt_params == other_num_opt_params
            }
            _ => false,
        }
    }
}

impl Eq for BaseObject {}

impl Hash for BaseObject {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            BaseObject::Null => {}
            BaseObject::True => true.hash(state),
            BaseObject::False => false.hash(state),
            BaseObject::Int(x) => x.hash(state),
            BaseObject::Float(x) => {
                unsafe {
                    // All I need is that the bytes of the float make it into the hasher.
                    // Floats cannot be Eq, so it doesn't really matter how accurate
                    // this step is.
                    mem::transmute::<f64, u64>(*x).hash(state);
                    'f'.hash(state);
                }
            }
            BaseObject::Tuple(v) => {
                v.hash(state);
            }
            BaseObject::Set(s) => {
                // This may be slower than XORing all element seeds together, but is much better for collisions
                // Maybe I didn't need to worry about this so much, and this might really only help if there
                // are a lot of sets being used in other sets or maps.
                let mut element_hashes = s.iter().map(|o| o.get_seed()).collect::<Vec<u64>>();
                element_hashes.sort();
                element_hashes.iter().for_each(|el| el.hash(state));
            }
            BaseObject::Closure {
                function,
                num_req_params,
                num_opt_params,
            } => {
                function.hash(state);
                num_req_params.hash(state);
                num_opt_params.hash(state);
            }
            _ => {}
        }
    }
}

#[derive(Clone, Debug)]
pub struct Object {
    base: Rc<BaseObject>,
    seed: Rc<OnceCell<u64>>,
}

impl Object {
    pub fn as_ref(&self) -> &BaseObject {
        self.base.as_ref()
    }

    pub fn get_seed(&self) -> u64 {
        *self
            .seed
            .get_or_try_init(|| {
                let mut tmp_hasher = DefaultHasher::new();
                self.base.hash(&mut tmp_hasher);
                Ok(tmp_hasher.finish()) as Result<u64, ()>
            })
            .unwrap()
    }

    pub fn make_range(range_start: i64, range_end: i64, step: Option<usize>, flag: u8) -> Self {
        let inclusive = flag & INCL_BIT != 0;
        let elements: Vec<Object> = if range_start <= range_end {
            // Normal range
            let high = if inclusive { range_end + 1 } else { range_end };
            (range_start..high)
                .step_by(step.unwrap_or(1))
                .map(|i| BaseObject::Int(i).wrap())
                .collect()
        } else {
            // Range starts from reverse
            let low = if inclusive { range_end } else { range_end + 1 };
            (low..range_start + 1)
                .rev()
                .step_by(step.unwrap_or(1))
                .map(|i| BaseObject::Int(i).wrap())
                .collect()
        };

        if flag & TUP_BASE == 0 {
            BaseObject::Set(HashSet::from_iter(elements)).wrap()
        } else {
            BaseObject::Tuple(elements).wrap()
        }
    }
}

impl ObjectOps for Object {
    fn is_null(&self) -> bool {
        self.base.is_null()
    }

    fn is_truthy(&self) -> bool {
        self.base.is_truthy()
    }

    fn truthy_convert(&self) -> Self {
        self.base.truthy_convert().wrap()
    }

    fn not(&self) -> Self {
        self.base.not().wrap()
    }

    fn negate(&self) -> Self {
        self.base.negate().wrap()
    }

    fn inner_int(&self) -> i64 {
        self.base.inner_int()
    }

    fn inner_fn(&self) -> (Executor, usize, usize) {
        self.base.inner_fn()
    }

    fn to_vec(&self) -> Vec<Object> {
        self.base.to_vec()
    }

    fn iter_kind(&self) -> IterKind {
        self.base.iter_kind()
    }

    fn to_s(&self) -> String {
        self.base.to_s()
    }

    fn to_debug_string(&self) -> String {
        self.base.to_debug_string()
    }
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        if let (Some(self_hash), Some(other_hash)) = (
            once_cell::unsync::OnceCell::<u64>::get(&self.seed),
            once_cell::unsync::OnceCell::<u64>::get(&other.seed),
        ) {
            self_hash == other_hash
        } else {
            self.base == other.base
        }
    }
}

impl Eq for Object {}

impl Hash for Object {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // I've made the decision that to facilitate caching my hashes, I must actually
        // hash twice, once. The flow is:
        // - If the "seed" value exists, we pass it to the hasher and we're done. It's only
        // a single u64, so it's pretty cheap.
        // - If the "seed" value doesn't exist yet, we generate it first using a separate
        // hasher (which may be an expensive operation), saving the result of that hash as
        // our seed going forward. Then, we can proceed with passing that to our hasher.
        // `Object::get_seed()` will perform this initial hashing step once (hidden behind
        // a OnceCell).
        state.write_u64(self.get_seed());
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
    pub locked_values: Vec<Object>,
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
