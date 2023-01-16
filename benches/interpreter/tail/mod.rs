//! Special interpreter context and instruction for tail call based dispatch.

mod context;
mod instr;

pub use self::{context::ExecutionContext, instr::Instr};
