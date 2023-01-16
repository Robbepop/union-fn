type In = Option<i32>;
type Out = Result<i32, String>;

#[union_fn::union_fn]
trait Test {
    type Output = Out;

    fn f0(input: In) -> Self::Output {
        match input {
            Some(value) => Ok(value),
            None => Err("encountered None".into()),
        }
    }
}

fn main() {
    use union_fn::Call;

    assert_eq!(Test::f0(Some(42)).call(), Ok(42));
    assert_eq!(Test::f0(None).call(), Err(String::from("encountered None")));
}
