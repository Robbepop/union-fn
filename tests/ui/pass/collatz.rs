use union_fn::CallWithContext as _;

#[union_fn::union_fn]
trait Instruction {
    type Context = i64;

    fn check_done(value: &mut Self::Context) -> Control {
        if *value <= 1 {
            return Control::Done
        }
        Control::NotDone
    }

    fn step1(value: &mut Self::Context) -> Control {
        if *value % 2 == 0 {
            *value /= 2;
        }
        Control::NotDone
    }

    fn step2(value: &mut Self::Context) -> Control {
        if *value % 2 != 0 {
            *value = (*value * 3) + 1
        }
        Control::NotDone
    }
}

#[derive(Copy, Clone)]
pub enum Control {
    Done,
    NotDone,
}

fn run_collatz(program: &[Instruction], input: i64) -> i64 {
    let mut i = 0;
    let mut value = input;
    loop {
        match program[i].call(&mut value) {
            Control::Done => return value,
            Control::NotDone => ()
        }
        i += 1;
        i %= program.len();
    }
}

fn main() {
    let instrs = [
        Instruction::check_done(),
        Instruction::step1(),
        Instruction::check_done(),
        Instruction::step2(),
    ];
    assert_eq!(run_collatz(&instrs, 1), 1);
    assert_eq!(run_collatz(&instrs, 19), 1);
    assert_eq!(run_collatz(&instrs, 42), 1);
}
