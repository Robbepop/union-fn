fn main() {}

#[union_fn::union_fn]
trait Foo {
    type Output = i32;

    fn foo() {}
}
