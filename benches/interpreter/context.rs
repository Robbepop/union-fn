use super::Stack;
use wasmi_core::{TrapCode, UntypedValue};

/// Represents control flow after execution of an [`Instr`].
#[derive(Debug, Copy, Clone)]
pub enum Control {
    /// Continue with the next instruction at `ip`.
    Continue,
    /// Return from the current function.
    Return,
}

/// The execution state.
#[derive(Debug)]
pub struct ExecutionContext {
    ip: usize,
    pub stack: Stack,
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self {
            ip: 0,
            stack: Stack::new(100),
        }
    }
}

impl ExecutionContext {
    pub fn feed_inputs(&mut self, inputs: &[i64]) {
        for input in inputs {
            self.stack.push(UntypedValue::from(*input))
        }
    }

    pub fn ip(&self) -> usize {
        self.ip
    }

    /// Continues with the next instruction in the sequence.
    pub fn next_instr(&mut self) -> Result<Control, TrapCode> {
        self.ip += 1;
        Ok(Control::Continue)
    }

    /// Branches to another instruction using the given `offset` to the `ip`.
    pub fn goto_instr(&mut self, offset: isize) -> Result<Control, TrapCode> {
        self.ip += offset as usize;
        Ok(Control::Continue)
    }

    /// Executes a binary instruction on the [`Stack`] via `f`.
    pub fn execute_unary(
        &mut self,
        f: fn(UntypedValue) -> UntypedValue,
    ) -> Result<Control, TrapCode> {
        self.stack.eval(f);
        self.next_instr()
    }

    /// Executes a binary instruction on the [`Stack`] via `f`.
    pub fn execute_binary(
        &mut self,
        f: fn(UntypedValue, UntypedValue) -> UntypedValue,
    ) -> Result<Control, TrapCode> {
        self.stack.eval2(f);
        self.next_instr()
    }

    /// Executes a binary instruction on the [`Stack`] via `f`.
    pub fn try_execute_binary(
        &mut self,
        f: fn(UntypedValue, UntypedValue) -> Result<UntypedValue, TrapCode>,
    ) -> Result<Control, TrapCode> {
        self.stack.try_eval2(f)?;
        self.next_instr()
    }
}
