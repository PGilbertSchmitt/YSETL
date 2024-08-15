use std::rc::Rc;

pub trait ObjectOps {
    fn to_s(&self) -> String;
}

#[derive(PartialEq, Debug)]
pub enum BaseObject {
    Null,
    True,
    False,
    Int(i64),
    Float(f64),
    String(String),
}

impl BaseObject {
    pub fn wrap(self) -> Object {
        Object(Rc::new(self))
    }
}

impl ObjectOps for BaseObject {
    fn to_s(&self) -> String {
        match self {
            Self::Null => String::from("null"),
            Self::False => String::from("false"),
            Self::True => String::from("true"),
            Self::Int(x) => format!("i{x}"),
            Self::Float(x) => format!("f{x}"),
            Self::String(val) => format!("\"{val}\""),
        }
    }
}

#[derive(Clone)]
pub struct Object(Rc<BaseObject>);

impl Object {
    pub fn as_ref(&self) -> &BaseObject {
        self.0.as_ref()
    }
}

impl ObjectOps for Object {
    fn to_s(&self) -> String {
        self.0.to_s()
    }
}
