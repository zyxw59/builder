#![no_std]

use core::fmt;
use core::marker::PhantomData;

pub use builder_derive::Builder;

pub trait Builder: Sized {
    type Builder;

    fn builder() -> Self::Builder;
}

impl<T> Builder for T
where
    T: BuilderWithCallback<fn(Self) -> Self>,
{
    type Builder = <Self as BuilderWithCallback<fn(Self) -> Self>>::CallbackBuilder;

    fn builder() -> Self::Builder {
        Self::builder_with_callback((|this| this) as fn(Self) -> Self)
    }
}

pub trait BuilderWithCallback<F: Callback<Self>>: Sized {
    type CallbackBuilder;

    fn builder_with_callback(callback: F) -> Self::CallbackBuilder;
}

pub trait Callback<T> {
    type Output;

    fn callback(self, this: T) -> Self::Output;
}

impl<F, I, O> Callback<I> for F
where
    F: FnOnce(I) -> O,
{
    type Output = O;

    fn callback(self, this: I) -> Self::Output {
        self(this)
    }
}

pub struct NoData<T>(PhantomData<T>);

impl<T> NoData<T> {
    pub const fn new() -> Self {
        NoData(PhantomData)
    }
}

impl<T> fmt::Debug for NoData<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("NoData")
    }
}

impl<T> fmt::Display for NoData<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("NoData")
    }
}

impl<T> Default for NoData<T> {
    fn default() -> Self {
        NoData(PhantomData)
    }
}

pub trait OrDefault<T>
where
    T: Sized,
{
    fn or_default(self) -> T;
}

impl<T> OrDefault<T> for T
where
    T: Sized,
{
    fn or_default(self) -> T {
        self
    }
}

impl<T> OrDefault<T> for NoData<T>
where
    T: Default,
{
    fn or_default(self) -> T {
        T::default()
    }
}
