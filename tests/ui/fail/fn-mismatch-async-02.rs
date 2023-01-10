fn main() {}

#[union_fn::union_fn]
trait Foo {
    async fn foo() {}
    fn bar() {}
}
