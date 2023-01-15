use wasmi_core::{TrapCode, UntypedValue};

/// The value stack.
#[derive(Debug)]
pub struct Stack {
    /// The stack pointer.
    ///
    /// Points to the next free element.
    sp: usize,
    /// The values on the stack.
    values: Vec<UntypedValue>,
}

impl Stack {
    /// Creates a new [`Stack`] with the given initial `capacity`.
    ///
    /// # Note
    ///
    /// If the stack height grows larger than the `capacity` the
    /// operation will trigger a runtime panic.
    pub fn new(capacity: usize) -> Self {
        Self {
            sp: 0,
            values: vec![UntypedValue::default(); capacity],
        }
    }

    /// Returns the `n`-th value on the [`Stack`] from the bottom.
    pub fn get_nth(&self, n: usize) -> UntypedValue {
        self.values[n]
    }

    /// Sets the `n`-th value on the [`Stack`] from the bottom to the new `value`.
    pub fn set_nth(&mut self, n: usize, value: UntypedValue) {
        self.values[n] = value;
    }

    /// Push the `value` onto the [`Stack`].
    pub fn push(&mut self, value: UntypedValue) {
        self.values[self.sp] = value;
        self.sp += 1;
    }

    /// Pops the top most value from the [`Stack`] and returns it.
    pub fn pop(&mut self) -> UntypedValue {
        self.sp -= 1;
        self.values[self.sp]
    }

    /// Pops the two top most values from the [`Stack`] and returns them.
    pub fn pop2(&mut self) -> (UntypedValue, UntypedValue) {
        self.sp -= 2;
        let lhs = self.values[self.sp];
        let rhs = self.values[self.sp + 1];
        (lhs, rhs)
    }

    /// Pops the top most value `t` from the [`Stack`] and pushes back the result of `f(t)`.
    pub fn eval(&mut self, f: impl FnOnce(UntypedValue) -> UntypedValue) {
        let input = self.values[self.sp - 1];
        let result = f(input);
        self.values[self.sp - 1] = result;
    }

    /// Pops the two top most values `t0` and `t1` from the [`Stack`] and pushes back the result of `f(t0, t1)`.
    pub fn eval2(&mut self, f: impl FnOnce(UntypedValue, UntypedValue) -> UntypedValue) {
        self.sp -= 1;
        let rhs = self.values[self.sp];
        let lhs = self.values[self.sp - 1];
        let result = f(lhs, rhs);
        self.values[self.sp - 1] = result;
    }

    /// Pops the two top most values `t0` and `t1` from the [`Stack`] and pushes back the result of `f(t0, t1)`.
    pub fn try_eval2(
        &mut self,
        f: impl FnOnce(UntypedValue, UntypedValue) -> Result<UntypedValue, TrapCode>,
    ) -> Result<(), TrapCode> {
        self.sp -= 1;
        let rhs = self.values[self.sp];
        let lhs = self.values[self.sp - 1];
        let result = f(lhs, rhs)?;
        self.values[self.sp - 1] = result;
        Ok(())
    }

    /// Pops the three top most values `t0`,..`t2` from the [`Stack`] and pushes back the result of `f(t0,..t2)`.
    pub fn eval3(
        &mut self,
        f: impl FnOnce(UntypedValue, UntypedValue, UntypedValue) -> UntypedValue,
    ) {
        self.sp -= 2;
        let t2 = self.values[self.sp + 1];
        let t1 = self.values[self.sp];
        let t0 = self.values[self.sp - 1];
        let result = f(t0, t1, t2);
        self.values[self.sp - 1] = result;
    }
}
