use anyhow::{anyhow, Result};
use askama::Template;
use axum::{
    extract,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use rusqlite::Connection;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new("server=debug"))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app = Router::new()
        .route("/greet/:name", get(greet))
        .route("/", get(index))
        .route("/stores", get(stores));

    tracing::debug!("listening on http://0.0.0.0:3000");
    let x = axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

fn connection() -> Result<Connection> {
    let path = "data.sqlite";
    Ok(rusqlite::Connection::open(path)?)
}

async fn index() -> Result<impl IntoResponse, AppError> {
    Ok(Html(
        std::fs::read_to_string("templates/index.html").unwrap_or("Error".to_string()),
    ))
}
async fn stores() -> Result<impl IntoResponse, AppError> {
    let connection = connection()?;
    let mut stmt = connection.prepare("SELECT subchains.ChainId, subchains.SubchainId, ChainName, SubchainName, StoreName FROM Stores JOIN Subchains on Stores.chainId = Subchains.chainId AND Stores.subchainid = Subchains.subchainid")?;
    #[derive(Debug)]
    struct StoreRow {
        chain_id: i64,
        subchain_id: i64,
        chain_name: String,
        subchain_name: String,
        store_name: String,
    }
    let mut result = stmt.query(())?;
    let mut stores = Vec::new();
    while let Some(row) = result.next()? {
        stores.push(StoreRow {
            chain_id: row.get(0)?,
            subchain_id: row.get(1)?,
            chain_name: row.get(2)?,
            subchain_name: row.get(3)?,
            store_name: row.get(4)?,
        });
    }
    #[derive(Template)]
    #[template(path = "stores.html")]
    struct StoresTemplate {
        stores: Vec<StoreRow>,
    }
    let template = StoresTemplate { stores };

    Ok(HtmlTemplate(template))
}

async fn greet(extract::Path(name): extract::Path<String>) -> impl IntoResponse {
    let template = HelloTemplate { name };
    HtmlTemplate(template)
}

/* Error handling magic */
// Make our own error that wraps `anyhow::Error`.
struct AppError(anyhow::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

#[derive(Template)]
#[template(path = "hello.html")]
struct HelloTemplate {
    name: String,
}

struct HtmlTemplate<T>(T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error: {}", err),
            )
                .into_response(),
        }
    }
}
