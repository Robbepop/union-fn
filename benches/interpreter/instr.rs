use super::context::Control;
use super::context::ExecutionContext;
use core::num::NonZeroIsize;
use union_fn::union_fn;
use wasmi_core::TrapCode;
use wasmi_core::UntypedValue;

/// A branch offset.
#[derive(Debug, Copy, Clone)]
pub struct BranchOffset(NonZeroIsize);

impl BranchOffset {
    /// Creates a new [`BranchOffset`] from the given `value`.
    ///
    /// # Panics
    ///
    /// If `value` is equal to zero.
    pub fn new(value: isize) -> Self {
        BranchOffset(NonZeroIsize::new(value).expect("cannot have a branch offset of zero"))
    }

    /// Returns the inner non-zero `isize` value.
    pub fn into_inner(self) -> isize {
        self.0.get()
    }
}

impl core::fmt::Debug for Instr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LocalGet { n } => f.debug_struct("LocalGet").field("n", n).finish(),
            Self::LocalSet { n } => f.debug_struct("LocalSet").field("n", n).finish(),
            Self::LocalTee { n } => f.debug_struct("LocalTee").field("n", n).finish(),
            Self::Drop {} => f.debug_struct("Drop").finish(),
            Self::Select {} => f.debug_struct("Select").finish(),
            Self::Ret {} => f.debug_struct("Ret").finish(),
            Self::RetEqz {} => f.debug_struct("RetEqz").finish(),
            Self::Br { offset } => f.debug_struct("Br").field("offset", offset).finish(),
            Self::BrEqz { offset } => f.debug_struct("BrEqz").field("offset", offset).finish(),
            Self::Constant { value } => f.debug_struct("Constant").field("value", value).finish(),
            Self::Add {} => f.debug_struct("Add").finish(),
            Self::Sub {} => f.debug_struct("Sub").finish(),
            Self::Mul {} => f.debug_struct("Mul").finish(),
            Self::DivS {} => f.debug_struct("DivS").finish(),
            Self::DivU {} => f.debug_struct("DivU").finish(),
            Self::RemS {} => f.debug_struct("RemS").finish(),
            Self::RemU {} => f.debug_struct("RemU").finish(),
            Self::And {} => f.debug_struct("And").finish(),
            Self::Or {} => f.debug_struct("Or").finish(),
            Self::Xor {} => f.debug_struct("Xor").finish(),
            Self::Rotl {} => f.debug_struct("Rotl").finish(),
            Self::Rotr {} => f.debug_struct("Rotr").finish(),
            Self::Eq {} => f.debug_struct("Eq").finish(),
            Self::Ne {} => f.debug_struct("Ne").finish(),
            Self::Eqz {} => f.debug_struct("Eqz").finish(),
            Self::LtS {} => f.debug_struct("LtS").finish(),
            Self::LtU {} => f.debug_struct("LtU").finish(),
            Self::LeS {} => f.debug_struct("LeS").finish(),
            Self::LeU {} => f.debug_struct("LeU").finish(),
            Self::GeS {} => f.debug_struct("GeS").finish(),
            Self::GeU {} => f.debug_struct("GeU").finish(),
            Self::GtS {} => f.debug_struct("GtS").finish(),
            Self::GtU {} => f.debug_struct("GtU").finish(),
            Self::Clz {} => f.debug_struct("Clz").finish(),
            Self::Ctz {} => f.debug_struct("Ctz").finish(),
            Self::Popcnt {} => f.debug_struct("Popcnt").finish(),
        }
    }
}

/// Instructions of our "simple" benchmark interpreter.
///
/// # Note
///
/// We cannot make it too simple since otherwise the loop-switch based instruction
/// dispatch might profit from optimizations due to the low number of instructions
/// that are not realistic for actual interpreters.
#[union_fn]
pub trait Instr {
    type Context = ExecutionContext;
    type Output = Result<Control, TrapCode>;

    /// Executes `local.get` operation.
    fn local_get(ctx: &mut Self::Context, n: usize) -> Self::Output {
        let value = ctx.stack.get_nth(n);
        ctx.stack.push(value);
        ctx.next_instr()
    }

    /// Executes `local.set` operation.
    fn local_set(ctx: &mut Self::Context, n: usize) -> Self::Output {
        let value = ctx.stack.pop();
        ctx.stack.set_nth(n, value);
        ctx.next_instr()
    }

    /// Executes `local.tee` operation.
    fn local_tee(ctx: &mut Self::Context, n: usize) -> Self::Output {
        let value = ctx.stack.peek();
        ctx.stack.set_nth(n, value);
        ctx.next_instr()
    }

    /// Drops the top most value on the stack.
    fn drop(ctx: &mut Self::Context) -> Self::Output {
        ctx.stack.pop();
        ctx.next_instr()
    }

    /// Push a stack value based on the top most value `s` on the stack.
    ///
    /// - Pushes the 3rd top most value if `s == 0`
    /// - Pushes the 2nd top most value if `s != 0`
    fn select(ctx: &mut Self::Context) -> Self::Output {
        ctx.stack.eval3(
            |lhs, rhs, selector| {
                if i32::from(selector) != 0 {
                    lhs
                } else {
                    rhs
                }
            },
        );
        ctx.next_instr()
    }

    /// Return the current execution.
    fn ret(_ctx: &mut Self::Context) -> Self::Output {
        Ok(Control::Return)
    }

    /// Return the current execution if the top most value on the stack is equal to zero.
    fn ret_eqz(ctx: &mut Self::Context) -> Self::Output {
        if i32::from(ctx.stack.pop()) == 0 {
            Ok(Control::Return)
        } else {
            ctx.next_instr()
        }
    }

    /// Branch using the given `offset`.
    fn br(ctx: &mut Self::Context, offset: BranchOffset) -> Self::Output {
        ctx.goto_instr(offset.into_inner())
    }

    /// Branch using the given `offset` if the top most value on the stack is equal to zero.
    fn br_eqz(ctx: &mut Self::Context, offset: BranchOffset) -> Self::Output {
        if i32::from(ctx.stack.pop()) == 0 {
            ctx.goto_instr(offset.into_inner())
        } else {
            ctx.next_instr()
        }
    }

    /// Push a constant `value` to the stack.
    fn constant(ctx: &mut Self::Context, value: i64) -> Self::Output {
        ctx.stack.push(UntypedValue::from(value));
        ctx.next_instr()
    }

    /// Sum the two top most values on the stack and push the result.
    fn add(ctx: &mut Self::Context) -> Self::Output {
        ctx.execute_binary(UntypedValue::i64_add)
    }

    /// Subtract the two top most values on the stack and push the result.
    fn sub(ctx: &mut Self::Context) -> Self::Output {
        ctx.execute_binary(UntypedValue::i64_sub)
    }

    /// Multiply the two top most values on the stack and push the result.
    fn mul(ctx: &mut Self::Context) -> Self::Output {
        ctx.execute_binary(UntypedValue::i64_mul)
    }

    /// Divide the two top most values on the stack and push the result.
    fn div_s(ctx: &mut Self::Context) -> Self::Output {
        ctx.try_execute_binary(UntypedValue::i64_div_s)
    }

    /// Divide the two top most values on the stack and push the result.
    fn div_u(ctx: &mut Self::Context) -> Self::Output {
        ctx.try_execute_binary(UntypedValue::i64_div_u)
    }

    /// Divide the two top most values on the stack and push the result.
    fn rem_s(ctx: &mut Self::Context) -> Self::Output {
        ctx.try_execute_binary(UntypedValue::i64_rem_s)
    }

    /// Divide the two top most values on the stack and push the result.
    fn rem_u(ctx: &mut Self::Context) -> Self::Output {
        ctx.try_execute_binary(UntypedValue::i64_rem_u)
    }

    /// Bitwise-and the two top most values on the stack and push the result.
    fn and(ctx: &mut Self::Context) -> Self::Output {
        ctx.execute_binary(UntypedValue::i64_and)
    }

    /// Bitwise-or the two top most values on the stack and push the result.
    fn or(ctx: &mut Self::Context) -> Self::Output {
        ctx.execute_binary(UntypedValue::i64_or)
    }

    /// Bitwise-xor the two top most values on the stack and push the result.
    fn xor(ctx: &mut Self::Context) -> Self::Output {
        ctx.execute_binary(UntypedValue::i64_xor)
    }

    /// Left-rotate the two top most values on the stack and push the result.
    fn rotl(ctx: &mut Self::Context) -> Self::Output {
        ctx.execute_binary(UntypedValue::i64_rotl)
    }

    /// Right-rotate the two top most values on the stack and push the result.
    fn rotr(ctx: &mut Self::Context) -> Self::Output {
        ctx.execute_binary(UntypedValue::i64_rotr)
    }

    /// Compares the two top most values via the `eq` comparison and push the result.
    fn eq(ctx: &mut Self::Context) -> Self::Output {
        ctx.execute_binary(UntypedValue::i64_eq)
    }

    /// Compares the two top most values via the `ne` comparison and push the result.
    fn ne(ctx: &mut Self::Context) -> Self::Output {
        ctx.execute_binary(UntypedValue::i64_ne)
    }

    /// Computes the top most value via `eqz` and push the result.
    fn eqz(ctx: &mut Self::Context) -> Self::Output {
        ctx.execute_unary(UntypedValue::i64_eqz)
    }

    /// Compares the two top most values via the `lt_s` comparison and push the result.
    fn lt_s(ctx: &mut Self::Context) -> Self::Output {
        ctx.execute_binary(UntypedValue::i64_lt_s)
    }

    /// Compares the two top most values via the `lt_u` comparison and push the result.
    fn lt_u(ctx: &mut Self::Context) -> Self::Output {
        ctx.execute_binary(UntypedValue::i64_lt_u)
    }

    /// Compares the two top most values via the `le_s` comparison and push the result.
    fn le_s(ctx: &mut Self::Context) -> Self::Output {
        ctx.execute_binary(UntypedValue::i64_le_s)
    }

    /// Compares the two top most values via the `le_u` comparison and push the result.
    fn le_u(ctx: &mut Self::Context) -> Self::Output {
        ctx.execute_binary(UntypedValue::i64_le_u)
    }

    /// Compares the two top most values via the `ge_s` comparison and push the result.
    fn ge_s(ctx: &mut Self::Context) -> Self::Output {
        ctx.execute_binary(UntypedValue::i64_ge_s)
    }

    /// Compares the two top most values via the `ge_u` comparison and push the result.
    fn ge_u(ctx: &mut Self::Context) -> Self::Output {
        ctx.execute_binary(UntypedValue::i64_ge_u)
    }

    /// Compares the two top most values via the `gt_s` comparison and push the result.
    fn gt_s(ctx: &mut Self::Context) -> Self::Output {
        ctx.execute_binary(UntypedValue::i64_gt_s)
    }

    /// Compares the two top most values via the `gt_u` comparison and push the result.
    fn gt_u(ctx: &mut Self::Context) -> Self::Output {
        ctx.execute_binary(UntypedValue::i64_gt_u)
    }

    /// Computes the leading zeros for the top most value on the stack and push the result.
    fn clz(ctx: &mut Self::Context) -> Self::Output {
        ctx.execute_unary(UntypedValue::i64_clz)
    }

    /// Computes the trailing zeros for the top most value on the stack and push the result.
    fn ctz(ctx: &mut Self::Context) -> Self::Output {
        ctx.execute_unary(UntypedValue::i64_ctz)
    }

    /// Computes the pop count for the top most value on the stack and push the result.
    fn popcnt(ctx: &mut Self::Context) -> Self::Output {
        ctx.execute_unary(UntypedValue::i64_popcnt)
    }
}
