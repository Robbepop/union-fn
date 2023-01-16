#[union_fn::union_fn]
trait Test {
    fn sum0() -> i32 {
        0
    }
    fn sum1(p0: i32) -> i32 {
        p0
    }
    fn sum2(p0: i32, p1: i32) -> i32 {
        p0 + p1
    }
    fn sum3(p0: i32, p1: i32, p2: i32) -> i32 {
        p0 + p1 + p2
    }
    fn sum4(p0: i32, p1: i32, p2: i32, p3: i32) -> i32 {
        p0 + p1 + p2 + p3
    }
    fn sum5(p0: i32, p1: i32, p2: i32, p3: i32, p4: i32) -> i32 {
        p0 + p1 + p2 + p3 + p4
    }
    fn sum6(p0: i32, p1: i32, p2: i32, p3: i32, p4: i32, p5: i32) -> i32 {
        p0 + p1 + p2 + p3 + p4 + p5
    }
    fn sum7(p0: i32, p1: i32, p2: i32, p3: i32, p4: i32, p5: i32, p6: i32) -> i32 {
        p0 + p1 + p2 + p3 + p4 + p5 + p6
    }
    fn sum8(p0: i32, p1: i32, p2: i32, p3: i32, p4: i32, p5: i32, p6: i32, p7: i32) -> i32 {
        p0 + p1 + p2 + p3 + p4 + p5 + p6 + p7
    }
    fn sum9(
        p0: i32,
        p1: i32,
        p2: i32,
        p3: i32,
        p4: i32,
        p5: i32,
        p6: i32,
        p7: i32,
        p8: i32,
    ) -> i32 {
        p0 + p1 + p2 + p3 + p4 + p5 + p6 + p7 + p8
    }
    fn sum10(
        p0: i32,
        p1: i32,
        p2: i32,
        p3: i32,
        p4: i32,
        p5: i32,
        p6: i32,
        p7: i32,
        p8: i32,
        p9: i32,
    ) -> i32 {
        p0 + p1 + p2 + p3 + p4 + p5 + p6 + p7 + p8 + p9
    }
    fn sum11(
        p0: i32,
        p1: i32,
        p2: i32,
        p3: i32,
        p4: i32,
        p5: i32,
        p6: i32,
        p7: i32,
        p8: i32,
        p9: i32,
        p10: i32,
    ) -> i32 {
        p0 + p1 + p2 + p3 + p4 + p5 + p6 + p7 + p8 + p9 + p10
    }
    fn sum12(
        p0: i32,
        p1: i32,
        p2: i32,
        p3: i32,
        p4: i32,
        p5: i32,
        p6: i32,
        p7: i32,
        p8: i32,
        p9: i32,
        p10: i32,
        p11: i32,
    ) -> i32 {
        p0 + p1 + p2 + p3 + p4 + p5 + p6 + p7 + p8 + p9 + p10 + p11
    }
    fn sum13(
        p0: i32,
        p1: i32,
        p2: i32,
        p3: i32,
        p4: i32,
        p5: i32,
        p6: i32,
        p7: i32,
        p8: i32,
        p9: i32,
        p10: i32,
        p11: i32,
        p12: i32,
    ) -> i32 {
        p0 + p1 + p2 + p3 + p4 + p5 + p6 + p7 + p8 + p9 + p10 + p11 + p12
    }
    fn sum14(
        p0: i32,
        p1: i32,
        p2: i32,
        p3: i32,
        p4: i32,
        p5: i32,
        p6: i32,
        p7: i32,
        p8: i32,
        p9: i32,
        p10: i32,
        p11: i32,
        p12: i32,
        p13: i32,
    ) -> i32 {
        p0 + p1 + p2 + p3 + p4 + p5 + p6 + p7 + p8 + p9 + p10 + p11 + p12 + p13
    }
    fn sum15(
        p0: i32,
        p1: i32,
        p2: i32,
        p3: i32,
        p4: i32,
        p5: i32,
        p6: i32,
        p7: i32,
        p8: i32,
        p9: i32,
        p10: i32,
        p11: i32,
        p12: i32,
        p13: i32,
        p14: i32,
    ) -> i32 {
        p0 + p1 + p2 + p3 + p4 + p5 + p6 + p7 + p8 + p9 + p10 + p11 + p12 + p13 + p14
    }
    fn sum16(
        p0: i32,
        p1: i32,
        p2: i32,
        p3: i32,
        p4: i32,
        p5: i32,
        p6: i32,
        p7: i32,
        p8: i32,
        p9: i32,
        p10: i32,
        p11: i32,
        p12: i32,
        p13: i32,
        p14: i32,
        p15: i32,
    ) -> i32 {
        p0 + p1 + p2 + p3 + p4 + p5 + p6 + p7 + p8 + p9 + p10 + p11 + p12 + p13 + p14 + p15
    }
}

fn main() {
    use union_fn::Call;

    assert_eq!(Test::sum0().call(), 0);
    assert_eq!(Test::sum1(1).call(), 1);
    assert_eq!(Test::sum2(1, 2).call(), 1 + 2);
    assert_eq!(Test::sum3(1, 2, 3).call(), 1 + 2 + 3);
    assert_eq!(Test::sum4(1, 2, 3, 4).call(), 1 + 2 + 3 + 4);
    assert_eq!(Test::sum5(1, 2, 3, 4, 5).call(), 1 + 2 + 3 + 4 + 5);
    assert_eq!(Test::sum6(1, 2, 3, 4, 5, 6).call(), 1 + 2 + 3 + 4 + 5 + 6);
    assert_eq!(Test::sum7(1, 2, 3, 4, 5, 6, 7).call(), 1 + 2 + 3 + 4 + 5 + 6 + 7);
    assert_eq!(Test::sum8(1, 2, 3, 4, 5, 6, 7, 8).call(), 1 + 2 + 3 + 4 + 5 + 6 + 7 + 8);
    assert_eq!(Test::sum9(1, 2, 3, 4, 5, 6, 7, 8, 9).call(), 1 + 2 + 3 + 4 + 5 + 6 + 7 + 8 + 9);
    assert_eq!(Test::sum10(1, 2, 3, 4, 5, 6, 7, 8, 9, 10).call(), 1 + 2 + 3 + 4 + 5 + 6 + 7 + 8 + 9 + 10);
    assert_eq!(Test::sum11(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11).call(), 1 + 2 + 3 + 4 + 5 + 6 + 7 + 8 + 9 + 10 + 11);
    assert_eq!(Test::sum12(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12).call(), 1 + 2 + 3 + 4 + 5 + 6 + 7 + 8 + 9 + 10 + 11 + 12);
    assert_eq!(Test::sum13(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13).call(), 1 + 2 + 3 + 4 + 5 + 6 + 7 + 8 + 9 + 10 + 11 + 12 + 13);
    assert_eq!(Test::sum14(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14).call(), 1 + 2 + 3 + 4 + 5 + 6 + 7 + 8 + 9 + 10 + 11 + 12 + 13 + 14);
    assert_eq!(Test::sum15(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15).call(), 1 + 2 + 3 + 4 + 5 + 6 + 7 + 8 + 9 + 10 + 11 + 12 + 13 + 14 + 15);
    assert_eq!(Test::sum16(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16).call(), 1 + 2 + 3 + 4 + 5 + 6 + 7 + 8 + 9 + 10 + 11 + 12 + 13 + 14 + 15 + 16);
}
