pub mod constant;
pub mod runtime;
pub mod runtime_pointer;

pub use constant::ConstantValue;
pub use runtime::RuntimeValue;
pub use runtime_pointer::{RuntimePointerMut, RuntimeReference};
