pub use union_fn_macro::union_fn;

#[union_fn]
trait Bytecode {
    type Context = u64;
    // type Output = Result<(), String>;
    
    fn i32_add(ctx: &mut Self::Context, rhs: i32) -> i32 { todo!() }
    fn select(ctx: &mut Self::Context, condition: i32, if_true: i32, if_false: i32) -> i32 { todo!() }
}

fn main() {}
