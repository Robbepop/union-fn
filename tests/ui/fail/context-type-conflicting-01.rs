fn main() {}

#[union_fn::union_fn]
trait Foo {
    type Context = i32;
    type Context = i64;
    
    fn foo(ctx: &mut Self::Context) {}
}
