extern crate dbi;
extern crate mysql_async as my;
extern crate futures;
extern crate tokio;

use dbi::*;
use futures::Future;
use std::sync::Arc;

#[derive(Debug, FromRow)]
pub struct User {
    id: i32,
    #[dbi(rename="name")]
    full_name: String
}

// pub type BoxedFuture<T> = Box<Future<Item=T, Error=my::errors::Error> + Send>;

#[dbi_trait(impl_for(new="UserDao"))]
pub trait UserDaoImpl {

    #[sql_query("SELECT * FROM users WHERE id = ?", use_named_params=true)]
    fn find_by_id(self, id: i32) -> BoxedFuture<Option<User>>;

    #[sql_query("SELECT * FROM users WHERE id = ?", mapper="|row| { let (id, full_name) = my::from_row_opt(row)?; Ok(User {id, full_name}) }")]
    fn find_by_id_faster(self, id: i32) -> BoxedFuture<Vec<User>>;

    #[sql_query("SELECT name FROM users")]
    fn find_all_names(self) -> BoxedFuture<Vec<User>>;

    #[sql_update("INSERT INTO users (name) VALUES (?)", get_last_insert_id=true)]
    fn create_user_named(self, name: String) -> BoxedFuture<Option<u64>>;

    #[sql_batch("INSERT INTO users (name, valid, email) VALUES (:name, :valid, :email)", use_named_params=true)]
    fn create_users(self, name: Vec<String>, valid: Vec<bool>, email: Vec<String>) -> BoxedFuture<()>;

}



fn main() {

    let (username, password, db) = (env!("DB_USERNAME"), env!("DB_PASSWORD"), env!("DB_NAME"));

    let pool = my::Pool::new(format!("mysql://{}:{}@localhost:3306/{}", username, password, db));
    
    // let future = UserDao(pool.get_conn()).find_by_id(1).and_then(|val| {
    let future = UserDao(pool.get_conn())
        .create_users(vec!["Jacob".into(), "Bess".into()], vec![false, true], vec!["kardeiz@gmail.com".into(), "bess@gmail.com".into()])
        .and_then(|val| {
        println!("{:?}", &val);
        pool.disconnect().map(|_| ())
    });

    let mut runtime = tokio::runtime::Runtime::new().unwrap();
    let _ = runtime.block_on(future).unwrap();
    runtime.shutdown_on_idle().wait().unwrap();

    // let pool1 = pool.clone();
    // let pool2 = pool.clone();

    // let future = Users(pool.get_conn()).find_by_id(1).and_then(move |val| {
    //     println!("{:?}", &val);
    //     Users(pool1.get_conn()).find_all_names()
    // }).and_then(move |val| {
    //     println!("{:?}", &val);
    //     pool2.disconnect().map(|_| ())
    // });



}