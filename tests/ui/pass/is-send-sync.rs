const _: () = {
    const fn is_send<T:Send>() {}
    const fn is_sync<T:Sync>() {}
    let _ = is_send::<Test>();
    let _ = is_sync::<Test>();
    let _ = is_send::<<Test as ::union_fn::IntoOpt>::Opt>();
    let _ = is_sync::<<Test as ::union_fn::IntoOpt>::Opt>();
};

#[union_fn::union_fn]
trait Test {
    fn f0() -> i32 { todo!() }
    fn f1(_p0: i32) -> i32 { todo!() }
    fn f2(_p0: i32, _p1: f32) -> i32 { todo!() }
    fn f3(_p0: i32, _p1: f32, _p2: i64) -> i32 { todo!() }
    fn f4(_p0: i32, _p1: f32, _p2: i64, _p3: f64) -> i32 { todo!() }
}

fn main() {}
