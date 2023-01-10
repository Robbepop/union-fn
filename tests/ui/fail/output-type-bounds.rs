fn main() {}

#[union_fn::union_fn]
trait Foo {
    type Output: Copy = i32;
    
    fn foo() -> Self::Output {}
}
