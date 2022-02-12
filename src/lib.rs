use core::marker::PhantomData;
use core::fmt;

pub use builder_derive::Builder;

pub trait Builder {
    type Builder;

    fn builder() -> Self::Builder;
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
