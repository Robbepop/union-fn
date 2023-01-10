fn main() {}

#[union_fn::union_fn]
trait Foo {
    type Output<T> = T;
    
    fn foo() -> Self::Output {}

    type Output = i64;
}
