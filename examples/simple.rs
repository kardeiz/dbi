extern crate dbi;
extern crate mysql_async as my;
extern crate futures;
extern crate tokio;

use dbi::sql_query;
use futures::Future;

#[derive(Debug)]
pub struct User {
    id: i32,
    name: String
}

pub trait UserDao: dbi::Dao {

    fn pos_mapper(tup: (i32, String)) -> User {
        let (id, name) = tup;
        User { id, name}
    }

    fn str_mapper(s: (String, )) -> String {
        s.0
    }

    #[sql_query("SELECT id, name FROM users WHERE id = ?", mapper="Self::pos_mapper")]
    fn find_by_id(&self, id: i32) -> Box<Future<Item=Option<User>, Error=my::errors::Error> + Send>;

    #[sql_query("SELECT name FROM users", mapper="Self::str_mapper")]
    fn find_all_names(&self) -> Box<Future<Item=Vec<String>, Error=my::errors::Error> + Send>;

}

pub struct Users {
    pool: my::Pool
}

impl dbi::Dao for Users {
    type Connection = my::futures::GetConn;
    fn connection(&self) -> Self::Connection {
        self.pool.get_conn()
    }
}

impl UserDao for Users {} 

pub fn run<F, T, U>(future: F) -> Result<T, U>
where
    F: Future<Item = T, Error = U> + Send + 'static,
    T: Send + 'static,
    U: Send + 'static,
{
    let mut runtime = tokio::runtime::Runtime::new().unwrap();
    let result = runtime.block_on(future);
    runtime.shutdown_on_idle().wait().unwrap();
    result
}

fn main() {
    // let mut runtime = tokio::runtime::Runtime::new().unwrap();

    let (username, password) = (env!("DB_USERNAME"), env!("DB_PASSWORD"));

    let dao = Users { pool: my::Pool::new(format!("mysql://{}:{}@localhost:3306/rdbi_test", username, password)) };

    // let future = dao.find_by_id(1).and_then(|val| {
    let future = dao.find_all_names().and_then(|val| {
        dao.pool.disconnect().map(|_| val)
    });

    let x = run(future).unwrap();

    println!("{:?}", &x);
}