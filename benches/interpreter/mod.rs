#![allow(dead_code)]

mod context;
mod instr;
mod stack;

use self::{
    context::{Control, ExecutionContext},
    instr::Instr,
    stack::Stack,
};
use union_fn::CallWithContext;
use wasmi_core::TrapCode;

/// Executes the given sequence of instructions and returns the result.
///
/// # Errors
///
/// If a trap occurs during execution.
pub fn execute(instrs: &[Instr], inputs: &[i64]) -> Result<i64, TrapCode> {
    let mut ctx = ExecutionContext::default();
    ctx.feed_inputs(inputs);
    while let Some(instr) = instrs.get(ctx.ip()) {
        match instr.call(&mut ctx)? {
            Control::Continue => (),
            Control::Return => return Ok(i64::from(ctx.stack.pop())),
        }
    }
    Err(TrapCode::UnreachableCodeReached)
}
