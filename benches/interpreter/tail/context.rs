use super::{super::Stack, Instr};
use union_fn::{CallWithContext, IntoOpt, UnionFn};
use wasmi_core::{TrapCode, UntypedValue};

/// The execution state.
pub struct ExecutionContext {
    ip: usize,
    instrs: Vec<InstrOpt>,
    pub stack: Stack,
}

type InstrOpt = <Instr as IntoOpt>::Opt;
type CallResult = <Instr as UnionFn>::Output;

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
        self.call_ip()?;
        let result: i64 = self.stack.pop().into();
        Ok(result)
    }

    /// Feed the following inputs to the [`ExecutionContext`].
    pub fn feed_inputs(&mut self, inputs: &[i64]) {
        self.ip = 0;
        self.stack.clear();
        for input in inputs {
            self.stack.push(UntypedValue::from(*input))
        }
    }

    /// Calls the instruction currently pointed at by the `ip`.
    #[inline]
    fn call_ip(&mut self) -> CallResult {
        // println!("{:?}\n", self.stack);
        debug_assert!(self.ip < self.instrs.len());
        // SAFETY: In debug mode we still test for the bounds.
        //         This unchecked_get is experimental to see whether
        //         Rust will then properly generate tail calls in
        //         some situations.
        unsafe { self.instrs.get_unchecked(self.ip).call(self) }
    }

    /// Continues with the next instruction in the sequence.
    #[inline]
    pub fn next_instr(&mut self) -> CallResult {
        self.ip = self.ip.wrapping_add(1);
        self.call_ip()
    }

    /// Branches to another instruction using the given `offset` to the `ip`.
    #[inline]
    pub fn goto_instr(&mut self, offset: isize) -> CallResult {
        self.ip = (self.ip as isize).wrapping_add(offset) as usize;
        self.call_ip()
    }

    /// Returns the top most value on the stack.
    #[inline]
    pub fn ret(&mut self) -> CallResult {
        Ok(())
    }

    /// Executes a binary instruction on the [`Stack`] via `f`.
    #[inline]
    pub fn execute_binary(
        &mut self,
        f: fn(UntypedValue, UntypedValue) -> UntypedValue,
    ) -> CallResult {
        self.stack.eval2(f);
        self.next_instr()
    }
}
