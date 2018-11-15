pub trait Connection: Sized {
    type Queryable: ::my::prelude::Queryable;
    type Inner: futures::Future<Item=Self::Queryable, Error=::my::errors::Error> + Send + 'static;
    fn connection(self) -> Self::Inner;
}

pub type BoxedFuture<T> = Box<futures::Future<Item=T, Error=::my::errors::Error> + Send>;

pub mod utils {

    use super::Connection;

    pub fn query<CF, Q, F, T>(conn_fut: CF, sql: &'static str, params: ::my::Params, mapper: F) 
        -> impl futures::Future<Item=crate::ResultSet<T>, Error=::my::errors::Error> + Send + 'static
        where 
            F: Fn(::my::Row) -> Result<T, ::my::FromRowError> + Send + Sync + 'static,
            CF: futures::Future<Item=Q, Error=::my::errors::Error> + Send + 'static,
            Q: ::my::prelude::Queryable,
            T: Send + 'static {

        use futures::{Future, Stream};
        use ::my::prelude::*;
       
        let rt = conn_fut.and_then(move |conn| {
            conn.prep_exec(sql, params)
        }).and_then(move |res| {
            res.reduce_and_drop(crate::ResultSet::None, move |mut acc, row| {
                acc.push(mapper(row))
            })
        }).and_then(|(_, val)| {
            futures::stream::iter(val)
                .map_err(|e| ::my::errors::ErrorKind::FromRow(e.0).into() )
                .fold(crate::ResultSet::None, |mut acc, val| {
                    futures::future::ok::<_, ::my::errors::Error>(acc.push(val))
                })
        });

        rt
    }

    pub fn update<CF, Q>(conn_fut: CF, sql: &'static str, params: ::my::Params) 
        -> impl futures::Future<Item=crate::ResultSet<u64>, Error=::my::errors::Error> + Send + 'static
        where 
            CF: futures::Future<Item=Q, Error=::my::errors::Error> + Send + 'static,
            Q: ::my::prelude::Queryable {

        use futures::{Future, Stream};
        use ::my::prelude::*;
       
        let rt = conn_fut.and_then(move |conn| {
            conn.drop_exec(sql, params)
        }).and_then(|conn| {
            let id = conn.get_last_insert_id()
                .map(|x| crate::ResultSet::One(x) )
                .unwrap_or_else(|| crate::ResultSet::None );
            futures::future::ok::<_, ::my::errors::Error>(id)
        });

        rt
    }

}


