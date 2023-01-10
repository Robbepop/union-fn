fn main() {}

#[union_fn::union_fn]
trait Foo {
    extern "C" fn foo() {}
    fn bar() {}
}
