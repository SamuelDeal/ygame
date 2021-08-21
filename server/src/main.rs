#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]

mod actor_msg;
mod client;
mod error;
mod game;
mod lobby;
mod log;
mod user;
mod utils;

use actix::{Actor, Addr};
use actix_web::http::StatusCode;
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use client::Client;
use lobby::Lobby;
use std::net::IpAddr;
use std::result::Result as StdResult;
use std::str::FromStr;

type Result<T> = StdResult<T, crate::error::Error>;

async fn index(req: HttpRequest, stream: web::Payload, srv: web::Data<Addr<Lobby>>) -> StdResult<HttpResponse, Error> {
	log::info!("New client connection");
	let client = Client::new(srv.get_ref().clone());
	ws::start(client, &req, stream)
}

async fn not_found(req: HttpRequest) -> actix_web::Result<HttpResponse> {
	log::debug!("404 not found request {}", req.path());
	Ok(HttpResponse::build(StatusCode::NOT_FOUND)
		.content_type("text/html; charset=utf-8")
		.body("<h1>Error 404</h1>"))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	std::env::set_var("RUST_LOG", "debug");
	std::env::set_var("RUST_BACKTRACE", "1");

	let matches = clap::App::new("Ybo game 1")
		.version("1.0")
		.author("Samuel D. <samuel.deal@gmail.com>")
		.about("Websocket game server for Ybo game 1")
		.arg(
			clap::Arg::with_name("port")
				.short("p")
				.long("port")
				.value_name("PORT")
				.help("Listen to given port (or YGAME_PORT env)")
				.takes_value(true)
				.default_value("8000")
				.required(true),
		)
		.arg(
			clap::Arg::with_name("listen")
				.short("l")
				.long("listen")
				.value_name("ADDR")
				.help("Socket address to listen to (or YGAME_LISTEN env)")
				.takes_value(true)
				.default_value("127.0.0.1"),
		)
		.arg(
			clap::Arg::with_name("v")
				.short("v")
				.multiple(true)
				.help("Sets the level of verbosity (max 3)"),
		)
		.get_matches();

	let address = match matches.is_present("listen") {
		true => matches.value_of("listen").unwrap().to_string(),
		false => match std::env::var("YGAME_LISTEN") {
			Ok(val) => val,
			Err(_e) => matches.value_of("listen").unwrap().to_string(),
		},
	};
	if let Err(e) = IpAddr::from_str(address.as_str()) {
		eprintln!("Invalid ip address: {}: {}", address, e);
		std::process::exit(1);
	}

	let port = match matches.is_present("port") {
		true => matches.value_of("port").unwrap().to_string(),
		false => match std::env::var("YGAME_PORT") {
			Ok(val) => val,
			Err(_e) => matches.value_of("port").unwrap().to_string(),
		},
	};
	match port.parse::<i32>() {
		Err(e) => {
			eprintln!("Invalid port: {}: {}", address, e);
			std::process::exit(1);
		}
		Ok(value) => match value {
			1..=65535 => (),
			_ => {
				eprintln!("Invalid port: {}", port);
				std::process::exit(1);
			}
		},
	};

	let verbose_count: u8 = std::cmp::min(8, matches.occurrences_of("v") + 1) as u8;
	log::set_verbosity(log::Verbosity::from(verbose_count));
	let bind_address: String = format!("{}:{}", address, port);
	log::info!("Listening to {}...", bind_address);

	let lobby = Lobby::new().start();
	let server_result = HttpServer::new(move || {
		App::new()
			.route("/websocket", web::get().to(index))
			.data(lobby.clone())
			.default_service(web::route().to(not_found))
	})
	.bind(bind_address)?
	.run()
	.await;

	log::info!("Game server end");

	server_result
}
