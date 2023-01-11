use union_fn::{CallWithContext as _, IntoOpt as _};

#[union_fn::union_fn]
trait Counter {
    type Context = i64;

    /// Bumps the value `by` the amount.
    fn bump_by(value: &mut Self::Context, by: i64) {
        *value += by;
    }

    /// Selects the values in `choices` depending on `value`.
    fn select(value: &mut Self::Context, choices: [i64; 4]) {
        *value = choices.get(*value as usize).copied().unwrap_or(0)
    }

    /// Divides the `value` by 2.
    fn div2(value: &mut Self::Context) {
        *value /= 2;
    }

    /// Resets the `value` to zero.
    fn reset(value: &mut Self::Context) {
        *value = 0;
    }
}

fn main() {
    let mut value = 0;

    Counter::bump_by(1).call(&mut value);
    assert_eq!(value, 1);

    Counter::bump_by(41).call(&mut value);
    assert_eq!(value, 42);

    Counter::div2().call(&mut value);
    assert_eq!(value, 21);

    Counter::reset().call(&mut value);
    assert_eq!(value, 0);

    let choices = [11, 22, 33, 44];
    let opt = Counter::select(choices).into_opt();
    for i in 0..5 {
        let mut value = i;
        opt.call(&mut value);
        assert_eq!(value, choices.get(i as usize).copied().unwrap_or(0));
    }
}
