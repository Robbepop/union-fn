use std::fmt::{self, Debug};
use wasmi_core::{TrapCode, UntypedValue};

/// The value stack.
pub struct Stack {
    /// The stack pointer.
    ///
    /// Points to the next free element.
    sp: usize,
    /// The values on the stack.
    values: Vec<UntypedValue>,
}

impl Debug for Stack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let values = &self.values[..self.sp];
        if let Some((first, rest)) = values.split_first() {
            write!(f, "[{}", i64::from(*first))?;
            for value in rest {
                write!(f, ", {}", i64::from(*value))?;
            }
            write!(f, "]")?;
            Ok(())
        } else {
            write!(f, "[]")
        }
    }
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

    /// Clears all values from the [`Stack`].
    ///
    /// # Note
    ///
    /// After this operation the [`Stack`] is equivalent to an empty [`Stack`].
    pub fn clear(&mut self) {
        self.sp = 0;
    }

    /// Returns the `n`-th value on the [`Stack`] from the bottom.
    pub fn get(&self, n: usize) -> UntypedValue {
        debug_assert!(n < self.values.len());
        // SAFETY: In debug mode we still test for the bounds.
        //         This unchecked_get is experimental to see whether
        //         Rust will then properly generate tail calls in
        //         some situations.
        unsafe { *self.values.get_unchecked(n) }
    }

    /// Sets the `n`-th value on the [`Stack`] from the bottom to the new `value`.
    pub fn set(&mut self, n: usize, value: UntypedValue) {
        debug_assert!(n < self.values.len());
        // SAFETY: In debug mode we still test for the bounds.
        //         This unchecked_get is experimental to see whether
        //         Rust will then properly generate tail calls in
        //         some situations.
        *unsafe { self.values.get_unchecked_mut(n) } = value;
    }

    /// Push the `value` onto the [`Stack`].
    pub fn push(&mut self, value: UntypedValue) {
        self.set(self.sp, value);
        self.sp += 1;
    }

    /// Peeks the top most value from the [`Stack`] and returns it.
    pub fn peek(&self) -> UntypedValue {
        self.get(self.sp - 1)
    }

    /// Pops the top most value from the [`Stack`] and returns it.
    pub fn pop(&mut self) -> UntypedValue {
        self.sp -= 1;
        self.get(self.sp)
    }

    /// Pops the top most value `t` from the [`Stack`] and pushes back the result of `f(t)`.
    #[inline]
    pub fn eval(&mut self, f: impl FnOnce(UntypedValue) -> UntypedValue) {
        let input = self.get(self.sp - 1);
        let result = f(input);
        self.set(self.sp - 1, result);
    }

    /// Pops the two top most values `t0` and `t1` from the [`Stack`] and pushes back the result of `f(t0, t1)`.
    #[inline]
    pub fn eval2(&mut self, f: impl FnOnce(UntypedValue, UntypedValue) -> UntypedValue) {
        self.sp -= 1;
        let rhs = self.get(self.sp);
        let lhs = self.get(self.sp - 1);
        let result = f(lhs, rhs);
        self.set(self.sp - 1, result);
    }

    /// Pops the two top most values `t0` and `t1` from the [`Stack`] and pushes back the result of `f(t0, t1)`.
    #[inline]
    pub fn try_eval2(
        &mut self,
        f: impl FnOnce(UntypedValue, UntypedValue) -> Result<UntypedValue, TrapCode>,
    ) -> Result<(), TrapCode> {
        self.sp -= 1;
        let rhs = self.get(self.sp);
        let lhs = self.get(self.sp - 1);
        let result = f(lhs, rhs)?;
        self.set(self.sp - 1, result);
        Ok(())
    }

    /// Pops the three top most values `t0`,..`t2` from the [`Stack`] and pushes back the result of `f(t0,..t2)`.
    #[inline]
    pub fn eval3(
        &mut self,
        f: impl FnOnce(UntypedValue, UntypedValue, UntypedValue) -> UntypedValue,
    ) {
        self.sp -= 2;
        let t2 = self.get(self.sp + 1);
        let t1 = self.get(self.sp);
        let t0 = self.get(self.sp - 1);
        let result = f(t0, t1, t2);
        self.set(self.sp - 1, result);
    }
}
