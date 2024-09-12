pub mod obj_bound_method;
pub mod obj_class;
pub mod obj_closure;
pub mod obj_function;
pub mod obj_instance;
pub mod obj_native;
pub mod obj_string;
pub mod obj_upvalue;

pub use obj_bound_method::ObjBoundMethod;
pub use obj_class::ObjClass;
pub use obj_closure::ObjClosure;
pub use obj_function::ObjFunction;
pub use obj_instance::ObjInstance;
pub use obj_native::ObjNative;
pub use obj_string::ObjString;
pub use obj_upvalue::ObjUpvalue;

use std::fmt::{Debug, Display};

#[derive(Debug, Clone, PartialEq)]
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::{chunk::Chunk, value::RuntimeValue};
    use std::{collections::HashMap, rc::Rc};

    #[test]
    fn i_can_make_string_objects() {
        let string = ObjString {
            chars: "String".into(),
        };
        assert_eq!(&string.chars, "String");
    }

    #[test]
    fn i_can_make_native_function_objects() {
        let native_fn_closure = |_: Vec<RuntimeValue>| RuntimeValue::Nil;
        let native_fn = ObjNative {
            function: native_fn_closure,
        };
        assert_eq!((native_fn.function)(vec![]), RuntimeValue::Nil);
    }

    #[test]
    fn it_prints_objects_correctly() {
        let function = Rc::new(ObjFunction {
            arity: 0,
            chunk: Chunk::default(),
            upvalue_count: 0,
            name: Some("function_name".into()),
        });
        let closure = Rc::new(ObjClosure {
            function: Rc::clone(&function),
            upvalues: vec![],
        });
        let bound_method = Rc::new(Object::BoundMethod(ObjBoundMethod {
            receiver: RuntimeValue::Nil,
            method: Rc::clone(&closure),
        }));
        let class_name = Rc::new(ObjString {
            chars: "ClassName".into(),
        });
        let class = Object::Class(ObjClass {
            name: class_name,
            methods: HashMap::new(),
        });
        let native_fn = Rc::new(Object::Native(ObjNative {
            function: |_| RuntimeValue::Nil,
        }));
        let upvalue = Object::UpValue(ObjUpvalue {
            location: Rc::new(RuntimeValue::Nil),
            closed: RuntimeValue::Nil,
            next: None,
        });

        let bound_method_display = format!("{bound_method}");
        let closure_display = format!("{closure}");
        let function_display = format!("{function}");
        let class_display = format!("{class}");
        let native_display = format!("{native_fn}");
        let upvalue_display = format!("{upvalue}");

        assert_eq!(bound_method_display, "<fn function_name>");
        assert_eq!(closure_display, "<fn function_name>");
        assert_eq!(function_display, "<fn function_name>");
        assert_eq!(class_display, "ClassName");
        assert_eq!(native_display, "<native fn>");
        assert_eq!(upvalue_display, "upvalue");
    }
}
