use union_fn::Call as _;

#[union_fn::union_fn]
trait MathFn {
    fn one() -> i32 { 1 }
    fn add(lhs: i32, rhs: i32) -> i32 { lhs + rhs }
    fn select(flag: bool, if_true: i32, if_false: i32) -> i32 {
        if flag { if_true } else { if_false }
    }
    fn at(index: usize, elements: [i32; 4]) -> i32 {
        elements.get(index).copied().unwrap_or(0)
    }
}

fn main() {
    assert_eq!(MathFn::one().call(), 1);
    assert_eq!(MathFn::add(1, 2).call(), 3);
    assert_eq!(MathFn::select(true, 10, 20).call(), 10);
    assert_eq!(MathFn::select(false, 10, 20).call(), 20);
    assert_eq!(MathFn::at(0, [11, 22, 33, 44]).call(), 11);
    assert_eq!(MathFn::at(1, [11, 22, 33, 44]).call(), 22);
    assert_eq!(MathFn::at(2, [11, 22, 33, 44]).call(), 33);
    assert_eq!(MathFn::at(3, [11, 22, 33, 44]).call(), 44);
    assert_eq!(MathFn::at(4, [11, 22, 33, 44]).call(), 0);
}
