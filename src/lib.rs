extern crate futures;
extern crate mysql_async as my;
extern crate dbi_macros;

use futures::Future;

pub use dbi_macros::sql_query;



pub trait Dao {
    type Connection: Future<Item=my::Conn, Error=my::errors::Error> + Send + 'static;
    fn connection(&self) -> Self::Connection;
}