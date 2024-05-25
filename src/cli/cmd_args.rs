use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = r###"led_matrix_daemon"###)]
pub struct CmdArgs {
    /// Path to the configuration file.
    #[arg(long, default_value = "/etc/led_matrix_daemon/config.toml")]
    pub config: PathBuf,
}
