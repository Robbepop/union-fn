use union_fn::{CallWithContext as _, IntoOpt as _, UnionFn};

/// Instructions for a basic stack-based calculator.
#[union_fn::union_fn]
trait Instr {
    type Context = Calculator;
    type Output = Result<Control, ErrorCode>;

    /// Pushes the `value` onto the stack.
    fn constant(ctx: &mut Self::Context, value: i64) -> Self::Output {
        ctx.push_value(value)?;
        ctx.next_instr()
    }

    /// Adds the top two values on the stack and push the result.
    fn add(ctx: &mut Self::Context) -> Self::Output {
        let rhs = ctx.pop_value()?;
        let lhs = ctx.pop_value()?;
        let result = lhs.wrapping_add(rhs);
        ctx.push_value(result)?;
        ctx.next_instr()
    }

    /// Multiplies the top two values on the stack and push the result.
    fn mul(ctx: &mut Self::Context) -> Self::Output {
        let rhs = ctx.pop_value()?;
        let lhs = ctx.pop_value()?;
        let result = lhs.wrapping_mul(rhs);
        ctx.push_value(result)?;
        ctx.next_instr()
    }

    /// Duplicates the value on the stack at given `depth` from top.
    ///
    /// # Note
    ///
    /// A `depth` of 0 refers to the top most value on the stack.
    fn dup(ctx: &mut Self::Context, depth: usize) -> Self::Output {
        let value = ctx.get_back(depth)?;
        ctx.push_value(value)?;
        ctx.next_instr()
    }

    /// Returns the top most value on the stack and ends calculation.
    fn ret(_ctx: &mut Self::Context) -> Self::Output {
        Ok(Control::Return)
    }
}

/// Tells the calculator to continue or stop processing instructions.
#[derive(Copy, Clone)]
pub enum Control {
    /// Execute the next instruction.
    Continue,
    /// End execution and return result.
    Return,
}

/// Errors that may occur during calculation.
#[derive(Debug, Copy, Clone)]
pub enum ErrorCode {
    /// Too many values on the stack.
    StackOverflow,
    /// Too few values on the stack.
    StackUnderflow,
    /// Unreachable code reached.
    UnreachableCodeReached,
}

/// Call-optimzied instruction type.
pub type InstrOpt = <Instr as UnionFn>::Opt;

/// The state of the calculator.
pub struct Calculator {
    stack: Vec<i64>,
    ip: usize,
}

impl Calculator {
    /// Creates a new [`Calculator`] with given stack `capacity`.
    pub fn new(capacity: usize) -> Self {
        Self {
            stack: Vec::with_capacity(capacity),
            ip: 0,
        }
    }

    /// Calculate the result of the instructions and returns the result.
    pub fn calculate(&mut self, instrs: &[InstrOpt], inputs: &[i64]) -> Result<i64, ErrorCode> {
        self.stack.clear();
        self.ip = 0;
        for input in inputs {
            self.push_value(*input)?;
        }
        while let Some(instr) = instrs.get(self.ip) {
            match instr.call(self)? {
                Control::Continue => (),
                Control::Return => {
                    return self.stack.pop().ok_or_else(|| ErrorCode::StackUnderflow)
                }
            }
        }
        Err(ErrorCode::UnreachableCodeReached)
    }

    /// Go to the next instruction.
    pub fn next_instr(&mut self) -> Result<Control, ErrorCode> {
        self.ip += 1;
        Ok(Control::Continue)
    }

    /// Returns the element at index `depth` from the back.
    ///
    /// # Note
    ///
    /// A `depth` of 0 refers to the top most value on the stack.
    pub fn get_back(&self, depth: usize) -> Result<i64, ErrorCode> {
        self.stack
            .iter()
            .rev()
            .nth(depth)
            .copied()
            .ok_or_else(|| ErrorCode::StackUnderflow)
    }

    /// Pops the top most value from the stack.
    ///
    /// # Errors
    ///
    /// If the stack is empty.
    pub fn pop_value(&mut self) -> Result<i64, ErrorCode> {
        self.stack.pop().ok_or_else(|| ErrorCode::StackUnderflow)
    }

    /// Pushes the `value` onto the stack.
    ///
    /// # Errors
    ///
    /// If the stack is already at its capacity.
    pub fn push_value(&mut self, value: i64) -> Result<(), ErrorCode> {
        if self.stack.len() == self.stack.capacity() {
            return Err(ErrorCode::StackOverflow);
        }
        self.stack.push(value);
        Ok(())
    }
}

fn main() {
    let mut calc = Calculator::new(10);
    // Instructions implementing the following formular: (2+3) * (2+3)
    let instrs1 = [
        Instr::constant(2),
        Instr::constant(3),
        Instr::add(),
        Instr::dup(0),
        Instr::mul(),
        Instr::ret(),
    ]
    .map(Instr::into_opt);
    let result = calc.calculate(&instrs1, &[]).unwrap();
    assert_eq!(result, (2 + 3) * (2 + 3));

    // (x + 1) * (y + 2)
    let instrs2 = [
        Instr::constant(2),
        Instr::add(),
        Instr::dup(1),
        Instr::constant(1),
        Instr::add(),
        Instr::mul(),
        Instr::ret(),
    ]
    .map(Instr::into_opt);
    for x in 0..10 {
        for y in 0..10 {
            let result = calc.calculate(&instrs2, &[x, y]).unwrap();
            assert_eq!(result, (x + 1) * (y + 2));
        }
    }
}
