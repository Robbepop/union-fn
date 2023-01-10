fn main() {}

#[union_fn::union_fn]
trait Foo {
    type Context = i32 where i32: Copy;
    
    fn foo(ctx: &mut Self::Context) {}
}
