use std::sync::Arc;

use actix_web::{web, App, HttpServer};
use clap::Parser;
use tokio::task::JoinSet;
use tokio::time::Instant;
use tracing::{debug, error, info, warn};

use crate::api::base64::{render_base64, render_base64_multiple};
use crate::api::files::render_files;
use crate::api::AppState;
use crate::cli::cmd_args::CmdArgs;
use crate::config::led_matrix_config::LedMatrixConfig;
use crate::config::lef_matrix_config_dto::LedMatrixConfigDto;
use crate::init::{get_systemd_socket, init_tracing};

mod api;
mod cli;
mod config;
mod hw;
mod init;

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    init_tracing()?;
    let cmd_args = CmdArgs::parse();

    let config = LedMatrixConfigDto::try_from(cmd_args.config.as_path())?;
    let config = Arc::new(LedMatrixConfig::try_from(config)?);
    config.log_led_matrix_versions()?;

    let unix_socket = config.unix_socket.clone();
    let listen_address = config.listen_address.clone();

    let (sender, receiver) = kanal::bounded_async(config.max_queue_size);
    let state = web::Data::new(AppState {
        sender,
        config: config.clone(),
    });

    let mut server = HttpServer::new(move || {
        App::new()
            .service(render_base64)
            .service(render_base64_multiple)
            .service(render_files)
            .app_data(state.clone())
    });

    match get_systemd_socket() {
        Ok(Some(listener)) => {
            server = server.listen_uds(listener)?;
            info!("Listening to the Uix Socket provided by systemd");
        }
        Ok(None) => {
            if let Some(socket_path) = unix_socket {
                info!(%socket_path, "Binding manually provided unix socket");
                server = server.bind_uds(socket_path.as_ref())?;
            }
        }
        Err(err) => {
            warn!(
                ?err,
                "Failed to bind to unix socket, falling back to provided socket path"
            );
            if let Some(socket_path) = unix_socket {
                info!(%socket_path, "Binding manually provided unix socket");
                server = server.bind_uds(socket_path.as_ref())?;
            }
        }
    }

    if let Some(listen_address) = listen_address {
        server = server.bind(listen_address.as_ref())?;
    }

    let mut join_set: JoinSet<anyhow::Result<()>> = JoinSet::new();
    let server = server.workers(config.num_http_workers).run();
    join_set.spawn(async move {
        server.await?;
        Ok(())
    });

    join_set.spawn(async move {
        loop {
            let render_task = receiver.recv().await?;
            let start = Instant::now();

            let config = config.clone();
            match render_task.render(config).await {
                Ok(_) => {
                    debug!("Rendered task in {:?}", start.elapsed());
                }
                Err(err) => {
                    error!(?err, "Failed to render task");
                }
            };
        }
    });

    if let Some(result) = join_set.join_next().await {
        let result = result?;
        warn!("Server stopped: {:?}", result);
    }

    Ok(())
}
