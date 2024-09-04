use std::{
    cell::Cell,
    collections::HashMap,
    fmt::{Debug, Display},
};

use crate::{chunk::Chunk, value::Value};

type NativeFn = Box<dyn Fn(Vec<Value<'static>>) -> Value<'static>>;

#[derive(Debug, PartialEq)]
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

impl<'a> Display for Object<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BoundMethod(b) => write!(f, "{}", b),
            Self::Class(c) => write!(f, "{}", c),
            Self::Function(fun) => write!(f, "{}", fun),
            Self::Instance(i) => write!(f, "{}", i),
            Self::Native(n) => write!(f, "{}", n),
            Self::String(s) => write!(f, "{}", s),
            Self::UpValue(u) => write!(f, "{}", u),
            Self::Closure(c) => write!(f, "{}", c),
        }
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct Obj<'a> {
    pub next: Cell<Option<&'a Object<'a>>>,
}

#[derive(Debug, PartialEq)]
pub struct ObjFunction<'a> {
    pub obj: Obj<'a>,
    pub arity: usize,
    pub upvalue_count: usize,
    pub chunk: Chunk<'a>,
    pub name: Option<&'a ObjString<'a>>,
}

impl<'a> Display for ObjFunction<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.name.is_none() {
            return write!(f, "<script>");
        }
        write!(f, "<fn {}>", self.name.unwrap().chars)
    }
}

#[derive(Debug, PartialEq)]
pub struct ObjString<'a> {
    pub obj: Obj<'a>,
    pub hash: u32,
    pub chars: String,
}

impl<'a> Display for ObjString<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.chars)
    }
}

#[derive(Debug, PartialEq)]
pub struct ObjUpvalue<'a> {
    pub obj: Obj<'a>,
    pub location: &'a Value<'a>,
    pub closed: Value<'a>,
    pub next: Option<&'a ObjUpvalue<'a>>,
}

impl<'a> Display for ObjUpvalue<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "upvalue")
    }
}

pub struct ObjNative<'a> {
    pub obj: Obj<'a>,
    pub function: NativeFn,
}

impl<'a> PartialEq for ObjNative<'a> {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

impl<'a> Debug for ObjNative<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ObjNative {{ obj: {:?}, function: <native fn>}}",
            self.obj
        )
    }
}

impl<'a> Display for ObjNative<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<native fn>")
    }
}

#[derive(Debug, PartialEq)]
pub struct ObjClosure<'a> {
    pub obj: Obj<'a>,
    pub function: &'a ObjFunction<'a>,
    pub upvalues: Vec<&'a ObjUpvalue<'a>>,
}

impl<'a> Display for ObjClosure<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.function)
    }
}

#[derive(Debug, PartialEq)]
pub struct ObjClass<'a> {
    pub obj: Obj<'a>,
    pub name: &'a ObjString<'a>,
    pub methods: HashMap<String, &'a ObjFunction<'a>>,
}

impl<'a> Display for ObjClass<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name.chars)
    }
}

#[derive(Debug, PartialEq)]
pub struct ObjInstance<'a> {
    pub obj: Obj<'a>,
    pub class: &'a ObjClass<'a>,
}

impl<'a> Display for ObjInstance<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} instance", self.class.name.chars)
    }
}

#[derive(Debug, PartialEq)]
pub struct ObjBoundMethod<'a> {
    pub obj: Obj<'a>,
    pub receiver: Value<'a>,
    pub method: &'a ObjClosure<'a>,
}

impl<'a> Display for ObjBoundMethod<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.method.function)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn i_can_make_string_objects() {
        let string1 = Object::String(ObjString {
            obj: Obj {
                next: Cell::new(None),
            },
            chars: "String".into(),
            hash: 1234,
        });
        let string2 = Object::String(ObjString {
            obj: Obj {
                next: Cell::new(Some(&string1)),
            },
            chars: "String2".into(),
            hash: 45321,
        });
        let string3 = Object::String(ObjString {
            obj: Obj {
                next: Cell::new(Some(&string2)),
            },
            chars: "String3".into(),
            hash: 182736,
        });

        if let Object::String(ref s) = string2 {
            s.obj.next.replace(None);
        };

        if let Object::String(ref s) = string3 {
            match s.obj.next.get() {
                None => panic!("Failed to find next object"),
                Some(st) => match st {
                    Object::String(string) => {
                        assert_eq!("String2", string.chars);
                    }
                    _ => panic!("Found wrong object"),
                },
            }
        }
    }

    #[test]
    fn i_can_make_native_function_objects() {
        let native_fn_closure = Box::new(|_: Vec<Value>| Value::Nil);
        let native_fn = ObjNative {
            obj: Obj {
                next: Cell::new(None),
            },
            function: native_fn_closure,
        };
        assert!(native_fn.obj.next.get().is_none());
        assert_eq!((native_fn.function)(vec![]), Value::Nil);
    }

    #[test]
    fn it_prints_objects_correctly() {
        let function_name = ObjString {
            obj: Obj::default(),
            chars: "function_name".into(),
            hash: 1234,
        };
        let function = ObjFunction {
            obj: Obj::default(),
            arity: 0,
            chunk: Chunk {
                code: vec![],
                constants: vec![],
                lines: vec![],
            },
            upvalue_count: 0,
            name: Some(&function_name),
        };
        let closure = ObjClosure {
            obj: Obj::default(),
            function: &function,
            upvalues: vec![],
        };
        let bound_method = Object::BoundMethod(ObjBoundMethod {
            obj: Obj::default(),
            receiver: Value::Nil,
            method: &closure,
        });
        let class_name = ObjString {
            obj: Obj::default(),
            chars: "ClassName".into(),
            hash: 1234,
        };
        let class = Object::Class(ObjClass {
            obj: Obj::default(),
            name: &class_name,
            methods: HashMap::new(),
        });
        let native_fn = Object::Native(ObjNative {
            obj: Obj::default(),
            function: Box::new(|_| Value::Nil),
        });
        let upvalue = Object::UpValue(ObjUpvalue {
            obj: Obj::default(),
            location: &Value::Nil,
            closed: Value::Nil,
            next: None,
        });

        let bound_method_display = format!("{bound_method}");
        let closure_display = format!("{closure}");
        let function_display = format!("{function}");
        let function_name_display = format!("{function_name}");
        let class_display = format!("{class}");
        let native_display = format!("{native_fn}");
        let upvalue_display = format!("{upvalue}");

        assert_eq!(bound_method_display, "<fn function_name>");
        assert_eq!(closure_display, "<fn function_name>");
        assert_eq!(function_display, "<fn function_name>");
        assert_eq!(function_name_display, "function_name");
        assert_eq!(class_display, "ClassName");
        assert_eq!(native_display, "<native fn>");
        assert_eq!(upvalue_display, "upvalue");
    }
}
