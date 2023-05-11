use battlemon_ethereum::{
    config::load_config,
    address::ToHex,
    startup::App,
    telemetry::{build_subscriber, init_subscriber},
};
use ethers::prelude::Address;
use eyre::{Result, WrapErr};

use once_cell::sync::Lazy;
use reqwest::{Client, Method, RequestBuilder, Response};

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();
    let subscriber = if std::env::var("TEST_LOG").is_ok() {
        build_subscriber(subscriber_name, default_filter_level, std::io::stdout)
    } else {
        build_subscriber(subscriber_name, default_filter_level, std::io::sink)
    };
    init_subscriber(subscriber).expect("Failed to init subscriber");
});

pub struct TestUser(Address);

impl TestUser {
    pub fn random() -> Self {
        Self(Address::random())
    }
}

impl TestUser {
    pub fn id(&self) -> String {
        self.0.to_hex()
    }
}

pub struct TestApp {
    pub address: String,
    // pub db_name: String,
    // pub db_pool: PgPool,
    pub test_user: TestUser,
}

impl TestApp {
    fn http_request_builder(
        &self,
        method: Method,
        path: &str,
        query: Option<&str>,
    ) -> RequestBuilder {
        let url = format!("http://{}/{path}", self.address);
        Client::new().request(method, url).query(&query)
    }

    fn http_get_builder(&self, path: &str, query: Option<&str>) -> RequestBuilder {
        self.http_request_builder(Method::GET, path, query)
    }

    pub async fn get(&self, path: &str, query: Option<&str>) -> Result<Response> {
        self.http_get_builder(path, query)
            .send()
            .await
            .wrap_err("Failed to make request")
    }
}

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);
    let mut config = load_config().expect("Failed to read configuration");
    // cfg.database.database_name = Uuid::new_v4().to_string();
    config.app.port = 0;
    let app = App::build(config)
        .await
        .expect("Failed to build app for testing");
    let address = format!("127.0.0.1:{}", app.port());
    tokio::spawn(app.run_until_stopped());
    // configure_database(&config.database).await;
    // let application = Application::build(config.clone())
    //     .await
    //     .expect("Failed to build application");
    // let address = format!("127.0.0.1:{}", application.port());
    // actix_web::rt::spawn(application.run_until_stopped());
    // let ret = TestApp {
    //     address,
    //     db_name: config.database.database_name.clone(),
    //     db_pool: get_connection_pool(&config.database),
    //     test_user: TestUser::generate(),
    // };
    // ret.test_user.store(&ret.db_pool).await;
    //
    // ret
    TestApp {
        test_user: TestUser::random(),
        address,
    }
}
