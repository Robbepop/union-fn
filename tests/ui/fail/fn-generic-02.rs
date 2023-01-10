fn main() {}

#[union_fn::union_fn]
trait Foo {
    fn foo() where i32: Copy {}
}
