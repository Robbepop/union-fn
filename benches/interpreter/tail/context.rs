use super::{super::{Stack}, Instr};
use union_fn::{IntoOpt, CallWithContext};
use wasmi_core::{TrapCode, UntypedValue};

/// The execution state.
pub struct ExecutionContext {
    ip: usize,
    instrs: Vec<InstrOpt>,
    pub stack: Stack,
}

type InstrOpt = <Instr as IntoOpt>::Opt;

impl Default for ExecutionContext {
    fn default() -> Self {
        Self {
            ip: 0,
            instrs: Vec::new(),
            stack: Stack::new(100),
        }
    }
}

impl ExecutionContext {
    /// Creates a new [`ExecutionContext`] for the given instructions.
    pub fn new(instrs: &[InstrOpt]) -> Self {
        Self {
            ip: 0,
            instrs: instrs.to_vec(),
            stack: Stack::new(100),
        }
    }

    /// Executes the [`ExecutionContext`] using the given `inputs`.
    pub fn execute(&mut self, inputs: &[i64]) -> Result<i64, TrapCode> {
        // println!("\nSTART\n");
        self.feed_inputs(inputs);
        self.instrs[0].call(self)
    }

    /// Calls the instruction currently pointed at by the `ip`.
    fn call_ip(&mut self) -> Result<i64, TrapCode> {
        // println!("{:?}\n", self.stack);
        self.instrs[self.ip].call(self)
    }

    /// Feed the following inputs to the [`ExecutionContext`].
    pub fn feed_inputs(&mut self, inputs: &[i64]) {
        self.ip = 0;
        self.stack.clear();
        for input in inputs {
            self.stack.push(UntypedValue::from(*input))
        }
    }

    /// Continues with the next instruction in the sequence.
    pub fn next_instr(&mut self) -> Result<i64, TrapCode> {
        self.ip += 1;
        self.call_ip()
    }

    /// Branches to another instruction using the given `offset` to the `ip`.
    pub fn goto_instr(&mut self, offset: isize) -> Result<i64, TrapCode> {
        self.ip = (self.ip as isize + offset) as usize;
        self.call_ip()
    }

    /// Returns the top most value on the stack.
    pub fn ret(&mut self) -> Result<i64, TrapCode> {
        let result: i64 = self.stack.pop().into();
        Ok(result)
    }

    /// Executes a binary instruction on the [`Stack`] via `f`.
    pub fn execute_binary(
        &mut self,
        f: fn(UntypedValue, UntypedValue) -> UntypedValue,
    ) -> Result<i64, TrapCode> {
        self.stack.eval2(f);
        self.next_instr()
    }
}
