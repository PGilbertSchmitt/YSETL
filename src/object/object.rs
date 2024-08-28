use std::rc::Rc;

use bytes::Bytes;

pub trait ObjectOps {
    fn is_truthy(&self) -> bool;
    fn is_null(&self) -> bool;
    fn truthy_convert(&self) -> Self;
    fn not(&self) -> Self;
    fn negate(&self) -> Self;
    fn inner_fn(&self) -> Function;

    fn to_s(&self) -> String;
    fn to_debug_string(&self) -> String;
}

#[derive(PartialEq, Debug)]
pub enum BaseObject {
    Null,
    True,
    False,
    Int(i64),
    Float(f64),
    String(String),
    Tuple(Vec<Object>),
    Closure(Function),
}

impl BaseObject {
    pub fn wrap(self) -> Object {
        Object(Rc::new(self))
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

    fn inner_fn(&self) -> Function {
        match &self {
            &BaseObject::Closure(function) => Function {
                ins: function.ins.clone(),
                num_locals: function.num_locals,
                num_req_params: function.num_req_params,
                num_opt_params: function.num_opt_params,
                locked_values: function.locked_values.clone(),
            },
            _ => panic!("Could not convert {self:?} into a function")
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
            // This could change if we also stored the function's string
            // along with the compliled data, but this is good enough for now
            Self::Closure(func) => format!(
                "fn({}, {}?) => [{} locals, {} bytes]",
                func.num_req_params,
                func.num_opt_params,
                func.num_locals,
                func.ins.len(),
            ),
        }
    }

    fn to_debug_string(&self) -> String {
        match self {
            Self::Null => String::from("null"),
            Self::False => String::from("false"),
            Self::True => String::from("true"),
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
            Self::Closure(func) => format!(
                "fn({}, {}?) => [{} locals, {} bytes]",
                func.num_req_params,
                func.num_opt_params,
                func.num_locals,
                func.ins.len(),
            )
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Object(Rc<BaseObject>);

impl Object {
    pub fn as_ref(&self) -> &BaseObject {
        self.0.as_ref()
    }
}

impl ObjectOps for Object {
    fn is_null(&self) -> bool {
        self.0.is_null()
    }

    fn is_truthy(&self) -> bool {
        self.0.is_truthy()
    }

    fn truthy_convert(&self) -> Self {
        self.0.truthy_convert().wrap()
    }

    fn not(&self) -> Self {
        self.0.not().wrap()
    }

    fn negate(&self) -> Self {
        self.0.negate().wrap()
    }

    fn inner_fn(&self) -> Function {
        self.0.inner_fn()
    }

    fn to_s(&self) -> String {
        self.0.to_s()
    }

    fn to_debug_string(&self) -> String {
        self.0.to_debug_string()
    }
}

// This is separated from the Closure enum type for 2 reasons:
// - Will probably box this later to keep the BaseObject size small
// - Custom `PartialEq` implementation (just in case)
#[derive(Debug)]
pub struct Function {
    pub ins: Bytes,
    pub num_locals: usize,
    pub num_req_params: usize,
    pub num_opt_params: usize,
    pub locked_values: Vec<Object>,
}

// TODO: Currently, equality between 2 functions checks that the instructions match
// (and that all the other inners match just in case). This could be pretty slow, so
// functions should be given unique IDs by the compiler.
// If function over-riding ever gets implemented, this equality check should also
// make sure that the overrides match.
impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        (
            &self.ins,
            self.num_locals,
            self.num_req_params,
            self.num_opt_params,
        ) == (
            &other.ins,
            self.num_locals,
            self.num_req_params,
            self.num_opt_params,
        )
    }
}
