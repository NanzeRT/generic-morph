use std::hint::black_box;

use frunk::{Generic, HList};

use generic_morph::{GenericMut, HListMut, Morph};

struct PC(u64);
struct Stack(Vec<u64>);
struct Memory([u64; 256]);
struct Done(bool);
struct StackTopTrace(Vec<Option<u64>>);

#[derive(GenericMut)]
struct CompState {
    pc: PC,
    stack: Stack,
    memory: Memory,
    done: Done,
    trace: StackTopTrace,
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

fn do_loop<S, I>(mut state: S, step: impl Fn(&mut S)) -> S
where
    S: for<'a> Morph<'a, HListMut!['a, Done], I>,
{
    loop {
        step(&mut state);
        let (done,) = state.morph();
        if done.0 {
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

fn make_inst_selector<S, I>(
    program: Vec<Instruction>,
    instrs: InstructionImpls<
        impl Fn(&mut S, u64),
        impl Fn(&mut S),
        impl Fn(&mut S),
        impl Fn(&mut S),
        impl Fn(&mut S),
        impl Fn(&mut S),
        impl Fn(&mut S),
    >,
) -> impl Fn(&mut S)
where
    S: for<'a> Morph<'a, HListMut!['a, PC, Done], I>,
{
    move |state: &mut S| {
        let (pc, _) = state.morph();
        let inst = pc.0;
        pc.0 += 1;
        use Instruction::*;
        match program.get(inst as usize) {
            None => {
                let (_, done) = state.morph();
                done.0 = true;
            }
            Some(&Push(lit)) => (instrs.push)(state, lit),
            Some(Add) => (instrs.add)(state),
            Some(Mul) => (instrs.mul)(state),
            Some(Store) => (instrs.store)(state),
            Some(Load) => (instrs.load)(state),
            Some(Jump) => (instrs.jump)(state),
            Some(JumpI) => (instrs.jumpi)(state),
        };
    }
}

fn push<S: for<'a> Morph<'a, HList!(&'a mut Stack), I>, I>(state: &mut S, lit: u64) {
    let (stack,) = state.morph();
    stack.0.push(lit);
}

#[derive(Generic)]
struct SubState<'a> {
    stack: &'a mut Stack,
    done: &'a mut Done,
    memory: &'a mut Memory,
}

fn add<S: for<'a> Morph<'a, HList!(&'a mut Stack, &'a mut Done, &'a mut Memory), I>, I>(
    state: &mut S,
) {
    let state: SubState = state.morph();
    match (state.stack.0.pop(), state.stack.0.pop()) {
        (Some(n1), Some(n2)) => state.stack.0.push(n1.wrapping_add(n2)),
        _ => state.done.0 = true,
    }
}

fn mul<S: for<'a> Morph<'a, HListMut!['a, Stack, Done], I>, I>(state: &mut S) {
    let (stack, done) = state.morph();
    match (stack.0.pop(), stack.0.pop()) {
        (Some(n1), Some(n2)) => stack.0.push(n1.wrapping_mul(n2)),
        _ => done.0 = true,
    }
}

fn store<S: for<'a> Morph<'a, HListMut!['a, Stack, Done, Memory], I>, I>(state: &mut S) {
    let state: SubState = state.morph();
    match (state.stack.0.pop(), state.stack.0.pop()) {
        (Some(addr), Some(data)) => match state.memory.0.get_mut(addr as usize) {
            Some(mem) => *mem = data,
            _ => state.done.0 = true,
        },
        _ => state.done.0 = true,
    }
}

fn load<S: for<'a> Morph<'a, HListMut!['a, Stack, Done, Memory], I>, I>(state: &mut S) {
    let (stack, done, memory) = state.morph();
    match stack.0.pop() {
        Some(addr) => match memory.0.get(addr as usize) {
            Some(&data) => stack.0.push(data),
            _ => done.0 = true,
        },
        _ => done.0 = true,
    }
}

fn jump<S: for<'a> Morph<'a, HListMut!['a, Stack, Done, PC], I>, I>(state: &mut S) {
    let (stack, done, pc) = state.morph();
    match stack.0.pop() {
        Some(addr) => pc.0 = addr,
        _ => done.0 = true,
    }
}

fn jumpi<S: for<'a> Morph<'a, HListMut!['a, Stack, Done, PC], I>, I>(state: &mut S) {
    let (stack, done, pc) = state.morph();
    match (stack.0.pop(), stack.0.pop()) {
        (Some(addr), Some(cond)) => {
            if cond != 0 {
                pc.0 = addr
            }
        }
        _ => done.0 = true,
    }
}

fn trace_stack_top<S: for<'a> Morph<'a, HListMut!['a, StackTopTrace, Stack], I>, I>(state: &mut S) {
    let (trace, stack) = state.morph();
    trace.0.push(stack.0.last().copied());
}

pub fn main() {
    let state = CompState {
        pc: PC(0),
        stack: Stack(vec![]),
        memory: Memory([0; 256]),
        done: Done(false),
        trace: StackTopTrace(vec![]),
    };

    let instructions = InstructionImpls {
        push,
        add,
        mul,
        store,
        load: |s: &mut _| {
            load(s);
            trace_stack_top(s);
        },
        jump,
        jumpi,
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

    // let inspect = trace_stack_top;

    let step = |state: &mut CompState| {
        inst_selector(state);
        // state = inspect(state);
        println!("{}: {:?}", state.pc.0, state.stack.0);
    };

    let state = do_loop(state, step);

    println!("tops trace: {:?}", state.trace.0);
}
