use super::super::BranchOffset;
use super::ExecutionContext;
use union_fn::union_fn;
use wasmi_core::{TrapCode, UntypedValue};

/// Instructions of our "simple" tail-call based benchmark interpreter.
///
/// # Note
///
/// We cannot make it too simple since otherwise the loop-switch based instruction
/// dispatch might profit from optimizations due to the low number of instructions
/// that are not realistic for actual interpreters.
#[union_fn]
#[derive(Debug)]
pub trait Instr {
    type Context = ExecutionContext;
    type Output = Result<(), TrapCode>;

    /// Executes `local.get` operation.
    fn local_get(ctx: &mut Self::Context, n: usize) -> Self::Output {
        // println!("local.get {n}");
        let value = ctx.stack.get(n);
        ctx.stack.push(value);
        ctx.next_instr()
    }

    /// Executes `local.tee` operation.
    fn local_tee(ctx: &mut Self::Context, n: usize) -> Self::Output {
        // println!("local.tee {n}");
        let value = ctx.stack.peek();
        ctx.stack.set(n, value);
        ctx.next_instr()
    }

    /// Return the current execution.
    fn ret(ctx: &mut Self::Context) -> Self::Output {
        // println!("ret");
        ctx.ret()
    }

    /// Branch using the given `offset`.
    fn br(ctx: &mut Self::Context, offset: BranchOffset) -> Self::Output {
        // println!("br {offset:?}");
        ctx.goto_instr(offset.into_inner())
    }

    /// Branch using the given `offset` if the top most value on the stack is equal to zero.
    fn br_eqz(ctx: &mut Self::Context, offset: BranchOffset) -> Self::Output {
        // println!("br_eqz {offset:?}");
        if i32::from(ctx.stack.pop()) == 0 {
            ctx.goto_instr(offset.into_inner())
        } else {
            ctx.next_instr()
        }
    }

    /// Push a constant `value` to the stack.
    fn constant(ctx: &mut Self::Context, value: i64) -> Self::Output {
        // println!("i64.contant {value}");
        ctx.stack.push(UntypedValue::from(value));
        ctx.next_instr()
    }

    /// Sum the two top most values on the stack and push the result.
    fn add(ctx: &mut Self::Context) -> Self::Output {
        // println!("i64.add");
        ctx.execute_binary(UntypedValue::i64_add)
    }

    /// Compares the two top most values via the `eq` comparison and push the result.
    fn ne(ctx: &mut Self::Context) -> Self::Output {
        // println!("i64.ne");
        ctx.execute_binary(UntypedValue::i64_ne)
    }
}
