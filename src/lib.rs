#![feature(trait_alias)]
#![feature(impl_trait_in_assoc_type)]

use std::ops::Add;

use frunk::{from_generic, hlist::Sculptor, Generic};

pub use morph_proc_macros::GenericMut;

pub trait CanMorphMut<H, I> = where
    Self: Sculptor<H, I>,
    H: Add<<Self as Sculptor<H, I>>::Remainder>;

pub trait CanMorph<'a, H, I> = GenericMut<ReprMut<'a>: CanMorphMut<H, I>> where Self: 'a;

pub trait Morph<'a, H, I> {
    fn morph<M>(&'a mut self) -> M
    where
        M: Generic<Repr = H>;
}

#[macro_export]
macro_rules! HListMut {
    ($lt:lifetime, $($types:ty),*) => [
        ::frunk::HList![$(&$lt mut $types),*,]
    ];
}

impl<'a, T, H, I> Morph<'a, H, I> for T
where
    T: CanMorph<'a, H, I>,
{
    fn morph<M>(&'a mut self) -> M
    where
        M: Generic<Repr = H>,
    {
        let (tar, _rem) = self.into().sculpt();
        from_generic(tar)
    }
}

pub trait GenericMut {
    type ReprMut<'a>
    where
        Self: 'a;
    fn into(&mut self) -> Self::ReprMut<'_>;
}
