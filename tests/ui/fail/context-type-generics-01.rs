fn main() {}

#[union_fn::union_fn]
trait Foo {
    type Context<T> = T;
    
    fn foo(ctx: &mut Self::Context) {}
}
