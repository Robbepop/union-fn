use union_fn::union_fn;

/// Instructions of our "simple" benchmark interpreter.
/// 
/// # Note
/// 
/// We cannot make it too simple since otherwise the loop-switch based instruction
/// dispatch might profit from optimizations due to the low number of instructions
/// that are not realistic for actual interpreters.
#[union_fn]
pub trait Instr {
    type Context = Executor;
    type Output = Result<Control, TrapCode>;

    /// Drops the top most value on the stack.
    fn drop(ctx: &mut Self::Context) -> Self::Output {
        todo!()
    }

    /// Push a stack value based on the top most value `s` on the stack.
    /// 
    /// - Pushes the 3rd top most value if `s == 0`
    /// - Pushes the 2nd top most value if `s != 0`
    fn select(ctx: &mut Self::Context) -> Self::Output {
        todo!()
    }

    /// Return the current execution.
    fn ret(ctx: &mut Self::Context) -> Self::Output {
        todo!()
    }

    /// Return the current execution if the top most value on the stack is equal to zero.
    fn ret_eqz(ctx: &mut Self::Context) -> Self::Output {
        todo!()
    }

    /// Branch using the given `offset`.
    fn br(ctx: &mut Self::Context, offset: isize) -> Self::Output {
        todo!()
    }

    /// Branch using the given `offset` if the top most value on the stack is equal to zero.
    fn br_eqz(ctx: &mut Self::Context, offset: isize) -> Self::Output {
        todo!()
    }

    /// Push a constant `value` to the stack.
    fn constant(ctx: &mut Self::Context, value: i64) -> Self::Output {
        todo!()
    }

    /// Sum the two top most values on the stack and push the result.
    fn add(ctx: &mut Self::Context) -> Self::Output {
        todo!()
    }

    /// Subtract the two top most values on the stack and push the result.
    fn sub(ctx: &mut Self::Context) -> Self::Output {
        todo!()
    }

    /// Multiply the two top most values on the stack and push the result.
    fn mul(ctx: &mut Self::Context) -> Self::Output {
        todo!()
    }

    /// Divide the two top most values on the stack and push the result.
    fn div_s(ctx: &mut Self::Context) -> Self::Output {
        todo!()
    }

    /// Divide the two top most values on the stack and push the result.
    fn div_u(ctx: &mut Self::Context) -> Self::Output {
        todo!()
    }

    /// Divide the two top most values on the stack and push the result.
    fn rem_s(ctx: &mut Self::Context) -> Self::Output {
        todo!()
    }

    /// Divide the two top most values on the stack and push the result.
    fn rem_u(ctx: &mut Self::Context) -> Self::Output {
        todo!()
    }

    /// Bitwise-and the two top most values on the stack and push the result.
    fn and(ctx: &mut Self::Context) -> Self::Output {
        todo!()
    }

    /// Bitwise-or the two top most values on the stack and push the result.
    fn or(ctx: &mut Self::Context) -> Self::Output {
        todo!()
    }

    /// Bitwise-xor the two top most values on the stack and push the result.
    fn xor(ctx: &mut Self::Context) -> Self::Output {
        todo!()
    }

    /// Left-rotate the two top most values on the stack and push the result.
    fn rotl(ctx: &mut Self::Context) -> Self::Output {
        todo!()
    }

    /// Right-rotate the two top most values on the stack and push the result.
    fn rotr(ctx: &mut Self::Context) -> Self::Output {
        todo!()
    }

    // eq
    // ne
    // nez
    // lt_s
    // lt_u
    // le_s
    // le_u
    // ge_s
    // ge_u
    // gt_s
    // gt_u

    // clz
    // ctz
    // popcnt
}

/// Represents control flow after execution of an [`Instr`].
#[derive(Debug, Copy, Clone)]
pub enum Control {
    /// Continue with the next instruction at `ip`.
    Continue,
    /// Return from the current function.
    Return,
}

/// An error code that may occur upon executing an [`Instr`].
#[derive(Debug, Copy, Clone)]
pub enum TrapCode {
    UnreachableCodeReached,
    DivisionByZero,
    IntegerOverflow,
    StackOverflow,
}

pub struct Executor {

}
