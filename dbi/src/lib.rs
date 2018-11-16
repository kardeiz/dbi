extern crate futures;
extern crate dbi_macros;

#[cfg(feature="mysql")]
extern crate mysql_async as my;

#[cfg(feature="mysql")]
pub mod mysql;

#[cfg(feature="mysql")]
pub use mysql::*;

pub mod exp {

    #[cfg(feature="mysql")]
    pub mod my {
        pub use ::my::*;
    }    

    pub mod futures {
        pub use futures::*;
    }
}

use futures::Future;
pub use dbi_macros::*;

#[derive(Debug, Clone)]
pub enum ResultSet<T> {
    None,
    One(T),
    Many(Vec<T>)
}

impl<T> ResultSet<T> {
    pub fn push(self, val: T) -> Self {
        match self {
            ResultSet::None => ResultSet::One(val),
            ResultSet::One(first) => ResultSet::Many(vec![first, val]),
            ResultSet::Many(mut vec) => {
                vec.push(val);
                ResultSet::Many(vec)
            }
        }
    }
}

impl<T> Into<()> for ResultSet<T> {
    fn into(self) -> () {
        ()
    }
}

impl<T> Into<Option<T>> for ResultSet<T> {
    fn into(self) -> Option<T> {
        match self {
            ResultSet::None => None,
            ResultSet::One(first) => Some(first),
            ResultSet::Many(vec) => vec.into_iter().next()
        }
    }
}

impl<T> Into<Vec<T>> for ResultSet<T> {
    fn into(self) -> Vec<T> {
        match self {
            ResultSet::None => vec![],
            ResultSet::One(first) => vec![first],
            ResultSet::Many(vec) => vec
        }
    }
}

impl<T> IntoIterator for ResultSet<T> {
    type Item = T;
    type IntoIter = ResultSetIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            ResultSet::None => ResultSetIter::None(::std::iter::empty()),
            ResultSet::One(first) => ResultSetIter::One(::std::iter::once(first)),
            ResultSet::Many(vec) => ResultSetIter::Many(vec.into_iter())
        }
    }
}

pub enum ResultSetIter<T> {
    None(::std::iter::Empty<T>),
    One(::std::iter::Once<T>),
    Many(::std::vec::IntoIter<T>)
}

impl<T> Iterator for ResultSetIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            ResultSetIter::None(ref mut i) => i.next(),
            ResultSetIter::One(ref mut i) => i.next(),
            ResultSetIter::Many(ref mut i) => i.next()
        }
    }
}



