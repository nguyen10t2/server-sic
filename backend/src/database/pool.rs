use std::sync::LazyLock;

use sqlx::{PgPool, postgres::PgPoolOptions};

use crate::configs::env::get_env;

pub static DB: LazyLock<PgPool> = LazyLock::new(|| {
    let db_url = get_env("DATABASE_URL", None)
        .expect("Chưa thiết lập biến môi trường DATABASE_URL và không có giá trị mặc định");

    PgPoolOptions::new()
        .max_connections(5)
        .connect_lazy(&db_url)
        .expect("Không thể kết nối đến cơ sở dữ liệu")
});
