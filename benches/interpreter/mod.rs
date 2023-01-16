mod context;
mod instr;
mod stack;
mod tail;

pub use self::instr::{BranchOffset, Instr};
pub use self::tail::{ExecutionContext as TailContext, Instr as TailInstr};
use self::{
    context::{Control, ExecutionContext},
    stack::Stack,
};
use union_fn::{CallWithContext, UnionFn};
use wasmi_core::TrapCode;

/// Executes the given sequence of instructions and returns the result.
///
/// # Errors
///
/// If a trap occurs during execution.
pub fn execute<I>(instrs: &[I], inputs: &[i64]) -> Result<i64, TrapCode>
where
    I: CallWithContext<Context = ExecutionContext>
        + UnionFn<Output = Result<Control, TrapCode>>
        + Copy
        + Clone,
{
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
