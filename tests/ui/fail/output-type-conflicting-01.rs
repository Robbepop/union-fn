fn main() {}

#[union_fn::union_fn]
trait Foo {
    type Output = i32;
    type Output = i64;

    fn foo() -> Self::Output {}
}
