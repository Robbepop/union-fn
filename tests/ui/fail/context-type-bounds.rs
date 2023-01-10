fn main() {}

#[union_fn::union_fn]
trait Foo {
    type Context: Copy = i32;
    
    fn foo(ctx: &mut Self::Context) {}
}
