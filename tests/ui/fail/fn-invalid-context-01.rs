fn main() {}

#[union_fn::union_fn]
trait Foo {
    type Context = i32;

    fn foo(ctx: &Self::Context) {}
}
