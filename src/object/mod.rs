pub mod obj_bound_method;
pub mod obj_class;
pub mod obj_closure;
pub mod obj_function;
pub mod obj_instance;
pub mod obj_native;
pub mod obj_string;
pub mod obj_upvalue;

use std::{
    fmt::{Debug, Display},
    rc::Rc,
};

use crate::object::{
    obj_bound_method::ObjBoundMethod, obj_class::ObjClass, obj_closure::ObjClosure,
    obj_function::ObjFunction, obj_instance::ObjInstance, obj_native::ObjNative,
    obj_string::ObjString, obj_upvalue::ObjUpvalue,
};

#[derive(Debug, PartialEq)]
pub enum Object {
    BoundMethod(ObjBoundMethod),
    Class(ObjClass),
    Function(ObjFunction),
    Instance(ObjInstance),
    Native(ObjNative),
    String(ObjString),
    UpValue(ObjUpvalue),
    Closure(ObjClosure),
}

impl Display for Object {
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
pub struct Obj {
    pub next: Option<Rc<Object>>,
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{chunk::Chunk, value::Value};
    use std::collections::HashMap;

    #[test]
    fn i_can_make_string_objects() {
        let string1 = Rc::new(Object::String(ObjString {
            obj: Obj { next: None },
            chars: "String".into(),
            hash: 1234,
        }));
        let string2 = Rc::new(Object::String(ObjString {
            obj: Obj {
                next: Some(string1),
            },
            chars: "String2".into(),
            hash: 45321,
        }));
        let mut string3 = Object::String(ObjString {
            obj: Obj {
                next: Some(string2),
            },
            chars: "String3".into(),
            hash: 182736,
        });

        if let Object::String(ref mut s) = &mut string3 {
            match s.obj.next.as_mut() {
                None => panic!("Failed to find next object"),
                Some(st) => match Rc::get_mut(st).unwrap() {
                    Object::String(string) => {
                        string.obj.next.take();
                    }
                    _ => panic!("Found wrong object"),
                },
            }
        };

        if let Object::String(ref s) = string3 {
            match &s.obj.next {
                None => panic!("Failed to find next object"),
                Some(st) => match &**st {
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
        let native_fn_closure = |_: Vec<Value>| Value::Nil;
        let native_fn = ObjNative {
            obj: Obj { next: None },
            function: native_fn_closure,
        };
        assert!(native_fn.obj.next.is_none());
        assert_eq!((native_fn.function)(vec![]), Value::Nil);
    }

    #[test]
    fn it_prints_objects_correctly() {
        let function_name = Rc::new(ObjString {
            obj: Obj::default(),
            chars: "function_name".into(),
            hash: 1234,
        });
        let function = Rc::new(ObjFunction {
            obj: Obj::default(),
            arity: 0,
            chunk: Chunk::default(),
            upvalue_count: 0,
            name: Some(Rc::clone(&function_name)),
        });
        let closure = Rc::new(ObjClosure {
            obj: Obj::default(),
            function: Rc::clone(&function),
            upvalues: vec![],
        });
        let bound_method = Rc::new(Object::BoundMethod(ObjBoundMethod {
            obj: Obj::default(),
            receiver: Value::Nil,
            method: Rc::clone(&closure),
        }));
        let class_name = Rc::new(ObjString {
            obj: Obj::default(),
            chars: "ClassName".into(),
            hash: 1234,
        });
        let class = Object::Class(ObjClass {
            obj: Obj::default(),
            name: class_name,
            methods: HashMap::new(),
        });
        let native_fn = Rc::new(Object::Native(ObjNative {
            obj: Obj::default(),
            function: |_| Value::Nil,
        }));
        let upvalue = Object::UpValue(ObjUpvalue {
            obj: Obj::default(),
            location: Rc::new(Value::Nil),
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
