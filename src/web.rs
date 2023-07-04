use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use color_eyre::Result as EyreResult;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::db::DomainCount;

pub async fn run(addr: Option<SocketAddr>, conn: Arc<Mutex<Connection>>) -> EyreResult<()> {
    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        .route("/ping", get(pong))
        // `POST /users` goes to `create_user`
        .route("/top_k", get(top_k))
        .with_state(conn);

    // run it with hyper
    let addr = addr.unwrap_or(SocketAddr::from(([127, 0, 0, 1], 3000)));
    tracing::debug!("listening on {}", &addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

async fn root() -> &'static str {
    "Hello, World!"
}

async fn pong() -> &'static str {
    "pong"
}

async fn top_k(
    // this argument tells axum to parse the request body
    // as JSON into a `CreateUser` type
    params: Query<TopKQuery>,
    conn: State<Arc<Mutex<Connection>>>,
) -> Result<(StatusCode, Json<TopKResponse>), (StatusCode, String)> {
    // insert your application logic here
    let k = params.k.unwrap_or(10);
    let conn = conn.lock().unwrap();
    match crate::db::top_k(&conn, k) {
        Ok(data) => Ok((StatusCode::OK, Json(TopKResponse { k, data }))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

#[derive(Deserialize, Serialize)]
struct TopKQuery {
    k: Option<u64>,
}

#[derive(Deserialize, Serialize)]
struct TopKResponse {
    k: u64,
    data: Vec<DomainCount>,
}
