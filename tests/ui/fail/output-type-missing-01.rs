fn main() {}

#[union_fn::union_fn]
trait Foo {
    type Output;

    fn foo() -> Self::Output {}
}
