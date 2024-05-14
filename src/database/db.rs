#[derive(rocket_db_pools::Database)]
#[database("sqlx")]
pub struct DbConn(sqlx::PgPool);

use rocket_db_pools::Initializer;

impl DbConn {
    pub fn init() -> Initializer<Self> {
        Initializer::new()
    }
}
