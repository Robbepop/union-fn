pub use union_fn::union_fn;
pub use union_fn::CallWithContext as _;

#[union_fn]
trait Instruction {
    type Context = ExecutionContext;
    type Output = Result<Control, TrapCode>;

    /// Returns a constant value.
    fn constant(ctx: &mut Self::Context, value: u64) -> Self::Output {
        ctx.push(value);
        ctx.next_instr()
    }

    /// Duplicates the value at `depth` on the stack.
    fn dup(ctx: &mut Self::Context, depth: usize) -> Self::Output {
        let dup = ctx.nth(depth)?;
        ctx.push(dup);
        ctx.next_instr()
    }

    /// Adds two `i64` values.
    fn i64_add(ctx: &mut Self::Context) -> Self::Output {
        let rhs = ctx.pop()? as i64;
        let lhs = ctx.pop()? as i64;
        let result = lhs.wrapping_add(rhs);
        ctx.push(result as u64);
        ctx.next_instr()
    }

    /// Divides the top two values.
    /// 
    /// # Errors
    /// 
    /// - If the divisor is equal to zero.
    /// - If the division results in an integer overflow.
    fn i64_div(ctx: &mut Self::Context) -> Self::Output {
        let rhs = ctx.pop()? as i64;
        if rhs == 0 {
            return Err(TrapCode::DivisionByZero);
        }
        let lhs = ctx.pop()? as i64;
        let result = match lhs.overflowing_div(rhs) {
            (result, false) => Ok(result),
            _ => Err(TrapCode::IntegerOverflow),
        }?;
        ctx.push(result as u64);
        ctx.next_instr()
    }

    /// Branches to the new instruction pointer.
    fn goto(ctx: &mut Self::Context, new_ip: usize) -> Self::Output {
        ctx.goto_instr(new_ip)
    }

    /// Returns from the execution.
    fn ret(_ctx: &mut Self::Context) -> Self::Output {
        Ok(Control::Return)
    }
}

pub struct ExecutionContext {
    instrs: Vec<Instruction>,
    ip: usize,
    stack: Vec<u64>,
}

impl ExecutionContext {
    pub fn exec_instr(&mut self) -> Result<Control, TrapCode> {
        self.instrs
            .get(self.ip)
            .copied()
            .ok_or_else(|| TrapCode::UnreachableCodeReached)?
            .call(self)
    }

    pub fn next_instr(&mut self) -> Result<Control, TrapCode> {
        self.ip += 1;
        Ok(Control::Continue)
    }

    pub fn goto_instr(&mut self, new_ip: usize) -> Result<Control, TrapCode> {
        self.ip += new_ip;
        Ok(Control::Continue)
    }

    pub fn pop(&mut self) -> Result<u64, TrapCode> {
        self.stack.pop().ok_or_else(|| TrapCode::StackUnderflow)
    }

    pub fn push(&mut self, value: u64) {
        self.stack.push(value)
    }

    pub fn nth(&self, depth: usize) -> Result<u64, TrapCode> {
        self.stack
            .iter()
            .rev()
            .nth(depth)
            .copied()
            .ok_or_else(|| TrapCode::StackUnderflow)
    }
}

#[derive(Copy, Clone)]
pub enum Control {
    Continue,
    Return,
}

#[derive(Copy, Clone)]
pub enum TrapCode {
    UnreachableCodeReached,
    IntegerOverflow,
    DivisionByZero,
    StackUnderflow,
}
