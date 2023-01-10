use union_fn::CallWithContext as _;

#[union_fn::union_fn]
trait Counter {
    type Context = i64;

    fn bump_one(ctx: &mut Self::Context) { *ctx += 1; }
    fn bump_by(ctx: &mut Self::Context, by: i64) { *ctx += by; }
    fn div2(ctx: &mut Self::Context) { *ctx /= 2; }
    fn reset(ctx: &mut Self::Context) { *ctx = 0; }
}

fn main() {
    let mut ctx = 0;

    Counter::bump_one().call(&mut ctx);
    assert_eq!(ctx, 1);

    Counter::bump_by(41).call(&mut ctx);
    assert_eq!(ctx, 42);

    Counter::div2().call(&mut ctx);
    assert_eq!(ctx, 21);

    Counter::reset().call(&mut ctx);
    assert_eq!(ctx, 0);
}
