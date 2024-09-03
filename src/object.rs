use std::collections::HashMap;

use crate::{chunk::Chunk, value::Value};

type NativeFn = Box<dyn Fn(Vec<Value>) -> Value>;

pub enum Object<'a> {
    BoundMethod(ObjBoundMethod<'a>),
    Class(ObjClass<'a>),
    Function(ObjFunction<'a>),
    Instance(ObjInstance<'a>),
    Native(ObjNative<'a>),
    String(ObjString<'a>),
    UpValue(ObjUpvalue<'a>),
    Closure(ObjClosure<'a>),
}

pub struct Obj<'a> {
    pub next: Option<&'a Object<'a>>,
}

pub struct ObjFunction<'a> {
    pub obj: Obj<'a>,
    pub arity: usize,
    pub upvalue_count: usize,
    pub chunk: Chunk<'a>,
    pub name: &'a ObjString<'a>,
}

pub struct ObjString<'a> {
    pub obj: Obj<'a>,
    pub hash: u32,
    pub chars: String,
}

pub struct ObjUpvalue<'a> {
    pub obj: Obj<'a>,
    pub location: &'a Value<'a>,
    pub closed: Value<'a>,
    pub next: Option<&'a ObjUpvalue<'a>>,
}

pub struct ObjNative<'a> {
    pub obj: Obj<'a>,
    pub function: NativeFn,
}

pub struct ObjClosure<'a> {
    pub obj: Obj<'a>,
    pub function: &'a ObjFunction<'a>,
    pub upvalues: Vec<&'a ObjUpvalue<'a>>,
}

pub struct ObjClass<'a> {
    pub obj: Obj<'a>,
    pub name: &'a ObjString<'a>,
    pub methods: HashMap<String, &'a ObjFunction<'a>>,
}

pub struct ObjInstance<'a> {
    pub obj: Obj<'a>,
    pub class: &'a ObjClass<'a>,
}

pub struct ObjBoundMethod<'a> {
    pub obj: Obj<'a>,
    pub receiver: Value<'a>,
    pub method: &'a ObjClosure<'a>,
}
