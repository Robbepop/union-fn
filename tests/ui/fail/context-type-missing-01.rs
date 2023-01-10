fn main() {}

#[union_fn::union_fn]
trait Foo {
    type Context;

    fn foo(ctx: &mut Self::Context) {}
}
