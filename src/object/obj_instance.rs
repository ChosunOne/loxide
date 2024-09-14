use crate::{object::ObjClass, value::RuntimeReference};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ObjInstance {
    pub class: RuntimeReference<ObjClass>,
}
