extern crate dbi;
extern crate mysql_async as my;
extern crate futures;
extern crate tokio;

use dbi::{sql_query, FromRow};
use futures::Future;
use std::sync::Arc;

#[derive(Debug, FromRow)]
pub struct User {
    id: i32,
    #[dbi(rename="name")]
    nameeeee: String
}

pub trait UserDao: dbi::Connection {

    #[sql_query("SELECT * FROM users WHERE id = ?")]
    fn find_by_id(self, id: i32) -> Box<Future<Item=Option<User>, Error=my::errors::Error> + Send>;

    #[sql_query("SELECT name FROM users")]
    fn find_all_names(self) -> Box<Future<Item=Vec<String>, Error=my::errors::Error> + Send>;

}

// pub struct Users(my::futures::GetConn);

// impl dbi::Connection for Users {
//     type Inner = my::Conn;
//     type Future = my::futures::GetConn;
//     fn connection(self) -> my::futures::GetConn {
//         self.0
//     }
// }

pub struct Users(my::Pool);

impl<'a> dbi::Connection for &'a Users {
    type Inner = my::Conn;
    type Future = my::futures::GetConn;
    fn connection(self) -> my::futures::GetConn {
        self.0.get_conn()
    }
}

impl<'a> UserDao for &'a Users {} 

fn main() {

    let (username, password) = (env!("DB_USERNAME"), env!("DB_PASSWORD"));

    let pool = my::Pool::new(format!("mysql://{}:{}@localhost:3306/rdbi_test", username, password));

    let users = Users(pool);

    let future = users.find_by_id(1).and_then(|val| {
        println!("{:?}", &val);
        users.0.disconnect().map(|_| ())
    });

    // let conn1 = pool.get_conn();
    // let conn2 = pool.get_conn();

    // let future = Users(conn1).find_by_id(1).and_then(|val| {
    //     println!("{:?}", &val);
    //     Users(conn2).find_all_names()
    // }).and_then(|val| {
    //     println!("{:?}", &val);
    //     pool.disconnect().map(|_| ())
    // });

    let mut runtime = tokio::runtime::Runtime::new().unwrap();
    let _ = runtime.block_on(future).unwrap();
    runtime.shutdown_on_idle().wait().unwrap();

}