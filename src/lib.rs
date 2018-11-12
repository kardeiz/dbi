extern crate futures;
extern crate mysql_async as my;
extern crate dbi_macros;

use futures::Future;

pub use dbi_macros::{sql_query, FromRow};

pub trait Connection: Sized {
    type Inner: my::prelude::Queryable;
    type Future: Future<Item=Self::Inner, Error=my::errors::Error> + Send + 'static;
    fn connection(self) -> Self::Future;
}

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


