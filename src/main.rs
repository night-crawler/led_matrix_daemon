#![feature(let_chains)]
#![feature(if_let_guard)]

use std::path::Path;

use actix_web::{App, get, HttpResponse, HttpServer, post, Responder, web};
use clap::Parser;
use tokio::sync::Mutex;
use tracing::info;

use crate::api::AppState;
use crate::api::base64::render_base64;
use crate::api::files::render_files;
use crate::cli::cmd_args::CmdArgs;
use crate::config::led_matrix_config::LedMatrixConfig;
use crate::config::lef_matrix_config_dto::LedMatrixConfigDto;
use crate::init::init_tracing;

mod cli;
mod config;
mod hw;
mod init;
mod api;

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    init_tracing()?;
    let cmd_args = CmdArgs::parse();

    let config = LedMatrixConfigDto::try_from(cmd_args.config.as_path())?;
    let config = LedMatrixConfig::try_from(config)?;

    let unix_socket = config.unix_socket.clone();
    let listen_address = config.listen_address.clone();

    let state = web::Data::new(AppState { config });

    let mut server = HttpServer::new(move || {
        App::new()
            .service(render_base64)
            .service(render_files)
            .app_data(state.clone())
    });

    if let Some(socket_path) = unix_socket {
        if Path::new(socket_path.as_ref()).exists() {
            info!(%socket_path, "Removing existing socket file");
            std::fs::remove_file(socket_path.as_ref())?;
        }
        server = server.bind_uds(socket_path.as_ref())?;
    };

    if let Some(listen_address) = listen_address {
        server = server.bind(listen_address.as_ref())?;
    }

    server.workers(1).run().await?;

    Ok(())
}
