use std::rc::Rc;

pub trait ObjectOps {
    fn is_truthy(&self) -> bool;
    fn is_null(&self) -> bool;
    fn truthy_convert(&self) -> Self;
    fn not(&self) -> Self;
    fn negate(&self) -> Self;
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

    fn to_s(&self) -> String {
        self.0.to_s()
    }

    fn to_debug_string(&self) -> String {
        self.0.to_debug_string()
    }
}
