use battlemon_ethereum::{
    address::ToHex,
    config::load_config,
    config::DatabaseConfig,
    startup::App,
    telemetry::{build_subscriber, init_subscriber},
};
use ethers::prelude::{rand, LocalWallet, Signature, Signer};
use eyre::{ensure, Result, WrapErr};

use once_cell::sync::Lazy;
use reqwest::{Client, Method, RequestBuilder, Response};
use serde::Serialize;
use serde_json::json;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;

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

pub struct TestApp {
    pub address: String,
    // pub db_name: String,
    pub db_pool: PgPool,
    pub wallet: LocalWallet,
}

impl TestApp {
    pub fn user_address(&self) -> String {
        self.wallet.address().to_hex()
    }
}

impl TestApp {
    fn http_request_builder<T: Serialize>(
        &self,
        method: Method,
        path: &str,
        query: Option<&str>,
        json: Option<T>,
    ) -> RequestBuilder {
        let url = format!("http://{}/{path}", self.address);
        let ret = Client::new().request(method, url).query(&query);

        match json {
            None => ret,
            Some(json) => ret.json(&json),
        }
    }

    fn http_get_builder(&self, path: &str, query: Option<&str>) -> RequestBuilder {
        self.http_request_builder::<()>(Method::GET, path, query, None)
    }

    fn http_post_builder<T: Serialize>(&self, path: &str, json: Option<T>) -> RequestBuilder {
        self.http_request_builder(Method::POST, path, None, json)
    }

    pub async fn get(&self, path: &str, query: Option<&str>) -> Result<Response> {
        let response = self
            .http_get_builder(path, query)
            .send()
            .await
            .wrap_err("Failed to make request")?;

        assert_success_status(response).await
    }

    pub async fn post<T: Serialize>(&self, path: &str, json: Option<T>) -> Result<Response> {
        let response = self
            .http_post_builder(path, json)
            .send()
            .await
            .wrap_err("Failed to make request")?;

        assert_success_status(response).await
    }

    pub async fn get_nonce_for_user(&self, user_id: &str) -> Result<Uuid> {
        let response = self
            .get(&format!("users/{user_id}/nonce"), None)
            .await
            .wrap_err("Failed to get nonce for user")?;

        response
            .json()
            .await
            .wrap_err("Failed to deserialize `Uuid` from `Value`")
    }

    pub async fn web3_auth(&self, signature: &str, user_id: &str) -> Result<String> {
        let json = json!({
            "signature": signature,
            "user_id": user_id,
        });

        self.post("web3_auth", Some(json))
            .await
            .wrap_err("Failed to authenticate user")?
            .json()
            .await
            .wrap_err("Failed to deserialize json from body")
    }

    pub async fn sign(&self, message: &str) -> Result<Signature> {
        self.wallet
            .sign_message(message)
            .await
            .wrap_err("Failed to sign message")
    }
}

pub async fn assert_success_status(response: Response) -> Result<Response> {
    let status = response.status();
    ensure!(
        status.is_success(),
        "The response's status isn't 200-299.\nThe status is `{status}`.\nError: {body}",
        body = response
            .text()
            .await
            .wrap_err("Failed to get body from response")?
    );

    Ok(response)
}

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);
    let mut config = load_config().expect("Failed to read configuration");
    config.db.db_name = Uuid::new_v4().to_string();
    let db_pool = configure_database(&config.db).await;
    config.app.port = 0;
    let app = App::build(config)
        .await
        .expect("Failed to build app for testing");
    let address = format!("127.0.0.1:{}", app.port());
    tokio::spawn(app.run_until_stopped());

    TestApp {
        db_pool,
        address,
        wallet: LocalWallet::new(&mut rand::thread_rng()),
    }
}

pub async fn configure_database(config: &DatabaseConfig) -> PgPool {
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres");

    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.db_name).as_str())
        .await
        .expect("Failed to create database");

    let db_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to Postgres");

    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .expect("Failed to migrate the database");

    db_pool
}
