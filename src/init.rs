use std::env;
use std::os::fd::FromRawFd;

use anyhow::bail;
use console_subscriber::ConsoleLayer;
use tracing::info;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

pub fn init_tracing() -> anyhow::Result<()> {
    let console_layer = ConsoleLayer::builder().with_default_env().spawn();
    let fmt_layer = tracing_subscriber::fmt::layer()
        .compact()
        .with_ansi(atty::is(atty::Stream::Stdout))
        .with_target(false);
    let filter_layer = EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new("info"))?;

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(console_layer)
        .init();

    Ok(())
}

pub fn get_systemd_socket() -> anyhow::Result<Option<std::os::unix::net::UnixListener>> {
    if let Ok(listen_fds) = env::var("LISTEN_FDS") {
        let listen_pid = env::var("LISTEN_PID")?.parse::<i32>()?;
        let listen_fd_names = env::var("LISTEN_FDNAMES")?;

        info!("LISTEN_FDNAMES={listen_fd_names}");
        info!("LISTEN_PID={listen_pid}; LISTEN_FDS={listen_fds}");

        let process_id = std::process::id();
        if listen_pid != process_id as i32 {
            bail!("LISTEN_PID={listen_pid} does not match the current process id: {process_id}");
        }

        let listen_fds: i32 = listen_fds.parse()?;
        if listen_fds != 1 {
            bail!("Invalid LISTEN_FDS: {listen_fds}");
        }

        let listener = unsafe { std::os::unix::net::UnixListener::from_raw_fd(3) };
        return Ok(Some(listener));
    }

    Ok(None)
}
