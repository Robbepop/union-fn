pub use func_union_macro::union_fn;

/// Trait implemented by a function union.
pub trait FuncUnion {
    /// The shared execution context.
    type Context;

    /// The common output type of all functions in the function union.
    type Output;

    /// The generated parameter union type shared by all functions in the function union.
    type Params;
}

#[derive(Debug, Copy, Clone)]
enum TrapCode {
    UnreachableCodeReached,
}

pub struct ExecutionContext {}

// given

#[union_fn]
trait Bytecode {
    type Context = ExecutionContext;
    type Output = Result<(), TrapCode>;
    
    fn i32_add(ctx: &mut Self::Context, rhs: i32) -> Self::Output { todo!() }
    fn select(ctx: &mut Self::Context, condition: i32, if_true: i32, if_false: i32) -> Self::Output { todo!() }
}

// the proc macro will expand to

// #[derive(Copy, Clone)]
// struct Bytecode {
//     handler: fn(&mut <Self as FuncUnion>::Context, BytecodeParams) -> Result<(), TrapCode>,
//     params: BytecodeParams,
// }

// impl Bytecode {
//     pub fn call(self, ctx: &mut <Self as FuncUnion>::Context) -> <Self as FuncUnion>::Output {
//         (self.handler)(ctx, self.params)
//     }
// }

// impl FuncUnion for Bytecode {
//     type Context = ExecutionContext;
//     type Output = Result<(), TrapCode>;
//     type Params = BytecodeParams;
// }

// #[derive(Copy, Clone)]
// union BytecodeParams {
//     i32_add: i32,
// }
