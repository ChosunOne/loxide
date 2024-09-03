use crate::object::Object;

pub enum Value<'a> {
    Bool(bool),
    Number(f64),
    Object(Box<Object<'a>>),
    Nil,
}
