use std::hint::black_box;

use frunk::Generic;

use generic_morph::{wrap, wrap_once, CanMorph};

struct PC(u64);
struct Stack(Vec<u64>);
struct Memory(Box<[u64; 256]>);
struct Done(bool);
struct StackTopTrace(Vec<Option<u64>>);

#[derive(Generic)]
struct CompState {
    pc: PC,
    stack: Stack,
    memory: Memory,
    done: Done,
    trace: StackTopTrace,
}

impl AsMut<PC> for CompState {
    fn as_mut(&mut self) -> &mut PC {
        &mut self.pc
    }
}

impl AsMut<Done> for CompState {
    fn as_mut(&mut self) -> &mut Done {
        &mut self.done
    }
}

#[allow(unused)]
enum Instruction {
    Push(u64),
    Add,
    Mul,
    Store,
    Load,
    Jump,
    JumpI,
}

#[inline(always)]
fn do_loop<S>(mut state: S, step: impl Fn(S) -> S) -> S
where
    S: AsMut<Done>,
{
    loop {
        state = step(state);
        if state.as_mut().0 {
            break state;
        }
    }
}

struct InstructionImpls<PUSH, ADD, MUL, STORE, LOAD, JUMP, JUMPI> {
    push: PUSH,
    add: ADD,
    mul: MUL,
    store: STORE,
    load: LOAD,
    jump: JUMP,
    jumpi: JUMPI,
}

#[inline(always)]
fn make_inst_selector<S>(
    program: Vec<Instruction>,
    instrs: InstructionImpls<
        impl Fn(S, u64) -> S,
        impl Fn(S) -> S,
        impl Fn(S) -> S,
        impl Fn(S) -> S,
        impl Fn(S) -> S,
        impl Fn(S) -> S,
        impl Fn(S) -> S,
    >,
) -> impl Fn(S) -> S
where
    S: AsMut<PC>,
    S: AsMut<Done>,
{
    move |mut state| {
        let pc: &mut PC = state.as_mut();
        let inst = pc.0;
        pc.0 += 1;
        use Instruction::*;
        state = match program.get(inst as usize) {
            None => {
                AsMut::<Done>::as_mut(&mut state).0 = true;
                state
            }
            Some(&Push(lit)) => (instrs.push)(state, lit),
            Some(Add) => (instrs.add)(state),
            Some(Mul) => (instrs.mul)(state),
            Some(Store) => (instrs.store)(state),
            Some(Load) => (instrs.load)(state),
            Some(Jump) => (instrs.jump)(state),
            Some(JumpI) => (instrs.jumpi)(state),
        };
        state
    }
}

#[inline(always)]
fn push<S, H, I1, I2>(state: S, lit: u64) -> S
where
    S: CanMorph<(Stack,), H, I1, I2>,
{
    wrap_once(move |(mut stack,): (Stack,)| {
        stack.0.push(lit);
        (stack,)
    })(state)
}

#[derive(Generic)]
struct SubState {
    stack: Stack,
    done: Done,
    memory: Memory,
}

#[inline(always)]
fn add(mut state: SubState) -> SubState {
    match (state.stack.0.pop(), state.stack.0.pop()) {
        (Some(n1), Some(n2)) => state.stack.0.push(n1.wrapping_add(n2)),
        _ => state.done.0 = true,
    }
    state
}

#[inline(always)]
fn mul((mut stack, mut done): (Stack, Done)) -> (Stack, Done) {
    match (stack.0.pop(), stack.0.pop()) {
        (Some(n1), Some(n2)) => stack.0.push(n1.wrapping_mul(n2)),
        _ => done.0 = true,
    }
    (stack, done)
}

#[inline(always)]
fn store(mut state: SubState) -> SubState {
    match (state.stack.0.pop(), state.stack.0.pop()) {
        (Some(addr), Some(data)) => match state.memory.0.get_mut(addr as usize) {
            Some(mem) => *mem = data,
            _ => state.done.0 = true,
        },
        _ => state.done.0 = true,
    }
    state
}

#[inline(always)]
fn load((mut stack, mut done, memory): (Stack, Done, Memory)) -> (Stack, Done, Memory) {
    match stack.0.pop() {
        Some(addr) => match memory.0.get(addr as usize) {
            Some(&data) => stack.0.push(data),
            _ => done.0 = true,
        },
        _ => done.0 = true,
    }
    (stack, done, memory)
}

#[inline(always)]
fn jump((mut stack, mut done, mut pc): (Stack, Done, PC)) -> (Stack, Done, PC) {
    match stack.0.pop() {
        Some(addr) => pc.0 = addr,
        _ => done.0 = true,
    }
    (stack, done, pc)
}

#[inline(always)]
fn jumpi((mut stack, mut done, mut pc): (Stack, Done, PC)) -> (Stack, Done, PC) {
    match (stack.0.pop(), stack.0.pop()) {
        (Some(addr), Some(cond)) => {
            if cond != 0 {
                pc.0 = addr
            }
        }
        _ => done.0 = true,
    }
    (stack, done, pc)
}

#[inline(always)]
fn trace_stack_top((mut trace, stack): (StackTopTrace, Stack)) -> (StackTopTrace, Stack) {
    trace.0.push(stack.0.last().copied());
    (trace, stack)
}

pub fn main() {
    let state = CompState {
        pc: PC(0),
        stack: Stack(vec![]),
        memory: Memory(vec![0; 256].into_boxed_slice().try_into().unwrap()),
        done: Done(false),
        trace: StackTopTrace(vec![]),
    };

    let instructions = InstructionImpls {
        push,
        add: wrap(add),
        mul: wrap(mul),
        store: wrap(store),
        load: |mut s| {
            s = wrap(load)(s);
            wrap(trace_stack_top)(s)
        },
        jump: wrap(jump),
        jumpi: wrap(jumpi),
    };

    use Instruction::*;
    let program = vec![
        Push(100),
        Push(0),
        Store,
        Push(1),
        Push(1),
        Store,
        Push(1),
        Push(1),
        Load,
        Push(2),
        Store,
        Push(1),
        Store,
        Push(1),
        Load,
        Push(2),
        Load,
        Add,
        Push(0),
        Load,
        Push(u64::MAX),
        Add,
        Push(0),
        Store,
        Push(0),
        Load,
        Push(7),
        JumpI,
    ];

    let program = black_box(program);

    let inst_selector = make_inst_selector(program, instructions);

    // let inspect = wrap(trace_stack_top);

    let step = |mut state: CompState| {
        state = inst_selector(state);
        // state = inspect(state);
        println!("{}: {:?}", state.pc.0, state.stack.0);
        state
    };

    let state = do_loop(state, step);

    println!("tops trace: {:?}", state.trace.0);
}
