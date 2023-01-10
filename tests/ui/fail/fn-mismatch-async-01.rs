fn main() {}

#[union_fn::union_fn]
trait Foo {
    fn foo() {}
    async fn bar() {}
}
