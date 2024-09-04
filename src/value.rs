use crate::object::Object;

#[derive(Debug, PartialEq)]
pub enum Value<'a> {
    Bool(bool),
    Number(f64),
    Object(Box<Object<'a>>),
    Nil,
}
