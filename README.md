# generic-morph

Aims to implement JS-ish "feature" to add new fields to object without altering existing logic.

End result intended to look like:
```rust
#[derive(GenericMut)]
struct CompState {
    vm: VmState,
    found_targets: FoundTargets,
    debug_info: DebugInfo,
    ...
}

#[derive(Generic)]
struct Components<'a> {
    vm: &'a mut VmState,
    found_targets: &'a mut FoundTargets,
}

fn foo<S, I>(state: &mut S)
where
    S: CanMorph<Components, I>,
{
    let conponents: Components = state.morph();
}

fn bar<S, I>(state: &mut S)
where
    S: CanMorph<(&mut VmState, &mut FoundTargets), I>,
{
    let (vm, found_targets): (&mut VmState, &mut FoundTargets) = state.morph();
}

fn main() {
    let mut state = CompState { ... };
    foo(&mut state);
    bar(&mut state);
}
```
