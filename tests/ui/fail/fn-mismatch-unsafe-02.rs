fn main() {}

#[union_fn::union_fn]
trait Foo {
    unsafe fn foo() {}
    fn bar() {}
}
