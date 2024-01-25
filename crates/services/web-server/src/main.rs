// region:    --- Modules

mod config;
mod error;
mod log;
mod web;

pub use self::error::{Error, Result};
use config::web_config;

use crate::web::middleware::res_map::response_map;
use crate::web::middleware::auth::{ctx_require, ctx_resolver};
use crate::web::middleware::req_stamp::req_stamp_resolver;
use axum::{middleware, Router};
use lib_core::_dev_utils;
use lib_core::model::ModelManager;
use tokio::net::TcpListener;
use tower_cookies::CookieManagerLayer;
use tracing::info;
use tracing_subscriber::EnvFilter;

// endregion: --- Modules

#[tokio::main]
async fn main() -> Result<()> {
	tracing_subscriber::fmt()
		.without_time() // For early local development.
		.with_target(false)
		.with_env_filter(EnvFilter::from_default_env())
		.init();

	// -- FOR DEV ONLY
	_dev_utils::init_dev().await;

	let mm = ModelManager::new().await?;

	// -- Define Routes
	let routes_rpc = web::routes::rpc::routes(mm.clone())
		.route_layer(middleware::from_fn(ctx_require));

	let routes_all = Router::new()
		.merge(web::routes::login::routes(mm.clone()))
		.nest("/api", routes_rpc)
		.layer(middleware::map_response(response_map))
		.layer(middleware::from_fn_with_state(mm.clone(), ctx_resolver))
		.layer(CookieManagerLayer::new())
		.layer(middleware::from_fn(req_stamp_resolver))
		.fallback_service(web::routes::static_routes::serve_dir());

	// region:    --- Start Server
	// Note: For this block, ok to unwrap.
	let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
	info!("{:<12} - {:?}\n", "LISTENING", listener.local_addr());
	axum::serve(listener, routes_all.into_make_service())
		.await
		.unwrap();
	// endregion: --- Start Server

	Ok(())
}
