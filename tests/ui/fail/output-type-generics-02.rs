fn main() {}

#[union_fn::union_fn]
trait Foo {
    type Output = i32 where i32: Copy;
    
    fn foo() -> Self::Output {}
}
