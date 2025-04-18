#![feature(trait_alias)]

use std::ops::Add;

use frunk::{from_generic, hlist::Sculptor, into_generic, Generic};

trait MapHListOnce<G> {
    fn map_hlist<List>(self, g: List) -> List
    where
        G: Generic<Repr = List>;
}

pub trait MapGenericOnce<Arg, Target, Indeces, Indeces2> {
    fn map_generic(self, g: Arg) -> Arg;
}

impl<G, T> MapHListOnce<G> for T
where
    T: FnOnce(G) -> G,
{
    fn map_hlist<H>(self, h: H) -> H
    where
        G: Generic<Repr = H>,
    {
        let g = from_generic(h);
        let g = self(g);
        into_generic(g)
    }
}

impl<Scul, Scul2, Indeces2, Indeces, Target, T, Arg, G> MapGenericOnce<Arg, G, Indeces, Indeces2>
    for T
where
    Scul: Sculptor<Target, Indeces>,
    T: MapHListOnce<G>,
    G: Generic<Repr = Target>,
    Arg: Generic<Repr = Scul>,
    Target: Add<Scul::Remainder, Output = Scul2>,
    Scul2: Sculptor<Scul, Indeces2>,
{
    fn map_generic(self, g: Arg) -> Arg {
        let (target, rem) = into_generic(g).sculpt();
        let target = self.map_hlist(target);
        from_generic((target + rem).sculpt().0)
    }
}

trait MapHListMut<G> {
    fn map_hlist<List>(&mut self, g: List) -> List
    where
        G: Generic<Repr = List>;
}

pub trait MapGenericMut<Arg, Target, Indeces, Indeces2> {
    fn map_generic(&mut self, g: Arg) -> Arg;
}

impl<G, T> MapHListMut<G> for T
where
    T: FnMut(G) -> G,
{
    fn map_hlist<H>(&mut self, h: H) -> H
    where
        G: Generic<Repr = H>,
    {
        let g = from_generic(h);
        let g = self(g);
        into_generic(g)
    }
}

impl<Scul, Scul2, Indeces2, Indeces, Target, T, Arg, G> MapGenericMut<Arg, G, Indeces, Indeces2>
    for T
where
    Scul: Sculptor<Target, Indeces>,
    T: MapHListMut<G>,
    G: Generic<Repr = Target>,
    Arg: Generic<Repr = Scul>,
    Target: Add<Scul::Remainder, Output = Scul2>,
    Scul2: Sculptor<Scul, Indeces2>,
{
    fn map_generic(&mut self, g: Arg) -> Arg {
        let (target, rem) = into_generic(g).sculpt();
        let target = self.map_hlist(target);
        from_generic((target + rem).sculpt().0)
    }
}

trait MapHList<G> {
    fn map_hlist<List>(&self, g: List) -> List
    where
        G: Generic<Repr = List>;
}

pub trait MapGeneric<Arg, Target, Indeces, Indeces2> {
    fn map_generic(&self, g: Arg) -> Arg;
}

impl<G, T> MapHList<G> for T
where
    T: Fn(G) -> G,
{
    fn map_hlist<H>(&self, h: H) -> H
    where
        G: Generic<Repr = H>,
    {
        let g = from_generic(h);
        let g = self(g);
        into_generic(g)
    }
}

impl<Scul, Scul2, Indeces2, Indeces, Target, T, Arg, G> MapGeneric<Arg, G, Indeces, Indeces2> for T
where
    Scul: Sculptor<Target, Indeces>,
    T: MapHList<G>,
    G: Generic<Repr = Target>,
    Arg: Generic<Repr = Scul>,
    Target: Add<Scul::Remainder, Output = Scul2>,
    Scul2: Sculptor<Scul, Indeces2>,
{
    fn map_generic(&self, g: Arg) -> Arg {
        let (target, rem) = into_generic(g).sculpt();
        let target = self.map_hlist(target);
        from_generic((target + rem).sculpt().0)
    }
}

pub fn wrap_once<S, T, I, I2, F>(fun: F) -> impl FnOnce(S) -> S
where
    F: MapGenericOnce<S, T, I, I2>,
{
    move |s| fun.map_generic(s)
}

pub fn wrap_mut<S, T, I, I2, F>(mut fun: F) -> impl FnMut(S) -> S
where
    F: MapGenericMut<S, T, I, I2>,
{
    move |s| fun.map_generic(s)
}

pub fn wrap<S, T, I, I2, F>(fun: F) -> impl Fn(S) -> S
where
    F: MapGeneric<S, T, I, I2>,
{
    move |s| fun.map_generic(s)
}

pub trait CanMorph<G, H, I, I2> = where
    Self: Generic,
    Self::Repr: Sculptor<H, I>,
    G: Generic<Repr = H>,
    H: Add<<Self::Repr as Sculptor<H, I>>::Remainder>,
    H::Output: Sculptor<Self::Repr, I2>;
