fn main() {}

#[union_fn::union_fn]
trait Foo {
    const fn foo() {}
    fn bar() {}
}
