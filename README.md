# dbi

A database interface for Rust, based loosely on the SQL Objects API of Java's [Jdbi](https://github.com/jdbi/jdbi/).

Uses Rust 1.30's procedural macros. Works on stable (current) Rust.

Currently only supports mysql (via [`mysql_async`](https://github.com/blackbeam/mysql_async)), but support for other database systems is planned.

## Usage

Define a trait with methods conforming to params and desired results for any number of queries, and pass in the SQL string in the method attributes:

```rust
#[dbi_trait(impl_for(new="UserDao"))]
pub trait UserDaoImpl {

    #[sql_query("SELECT * FROM users WHERE id = :id", use_named_params=true)]
    fn find_by_id(self, id: i32) -> Box<Future<Item=Option<User>, Error=my::errors::Error> + Send>;

    #[sql_query("SELECT * FROM users WHERE id = ?", mapper="|row| { let (id, full_name) = my::from_row_opt(row)?; Ok(User {id, full_name}) }")]
    fn find_by_id_faster(self, id: i32) -> Box<Future<Item=Option<User>, Error=my::errors::Error> + Send>;

    #[sql_query("SELECT name FROM users")]
    fn find_all_names(self) -> Box<futures::Future<Item=Vec<String>, Error=my::errors::Error> + Send>;

    #[sql_update("INSERT INTO users (name) VALUES (:name)", use_named_params=true)]
    fn create_user_named(self, name: String) -> Box<futures::Future<Item=Option<u64>, Error=my::errors::Error> + Send>;

}
```

This will create a "connection-like" newtype wrapper that you can use like:

```rust
let fut = UserDao(conn).find_by_id(2);
```

Trait methods must return a value that looks like `Box<Future<Item=R> ...>`, where `R` is an `Option<T>` or `Vec<T>` and T is a type that can be extracted from a database row.

Currently supports the following options:

* `use_named_params`: Uses the method's param names instead of position in formatting query params.
* `mapper`: use some other expression to map to a result type. Can be a closure or other method.

More features are planned, although this library will not provide all the options that Jdbi does.

## Changelog

* **0.3.0** 

    * Add `sql_batch`. Currently does not return any value (only works with a method that returns `Future<Item=()>,...>`). Similar to Jdbi, each param is passed in as its own container (e.g., each named param would have a method arg that looks like `name: Vec<String>`).