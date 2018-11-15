# dbi

A database interface for Rust, based loosely on [Jdbi](https://github.com/jdbi/jdbi/).

Define a trait with methods conforming to your params and desired results, and pass in the SQL string in the method attributes:

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

Trait methods must return a value that looks like `Box<Future<Item=R> ...>`, where `R` is a type that can be extracted from a database row.

Currently supports the following options:

* `use_named_params`: Uses the method's param names instead of position in formatting query params.
* `mapper`: use some other expression to map to a result type. Can be a closure or other method.

More features are planned, although this library will not provide all the options that Jdbi does.
