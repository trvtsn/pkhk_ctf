pub mod config {
    use once_cell::sync::Lazy;
    use std::env;

    pub static DATABASE_URL: Lazy<String> = Lazy::new(|| {
        env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env or environment")
    });

    pub static ADMIN_USERNAME: Lazy<String> = Lazy::new(|| {
        env::var("ADMIN_USERNAME").expect("ADMIN_EMAIL must be set in .env or environment")
    });

    pub static ADMIN_EMAIL: Lazy<String> = Lazy::new(|| {
        env::var("ADMIN_EMAIL").expect("ADMIN_EMAIL must be set in .env or environment")
    });

    pub static ADMIN_PASSWORD: Lazy<String> = Lazy::new(|| {
        env::var("ADMIN_PASSWORD").expect("ADMIN_PASSWORD must be set in .env or environment")
    });
}
