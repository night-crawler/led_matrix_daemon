#![feature(let_chains)]
#![feature(if_let_guard)]

use std::thread;

use clap::Parser;

use crate::cli::cmd_args::CmdArgs;
use crate::config::lef_matrix_config_dto::LedMatrixConfigDto;
use crate::hw::sync_impl::display_gray_image_cmd;

mod hw;
mod config;
mod cli;


fn q() {
    let h1 = thread::spawn(move || {
        display_gray_image_cmd("/dev/ttyACM0", "./test_data/img.png")
    });

    let h2 = thread::spawn(move || {
        display_gray_image_cmd("/dev/ttyACM1", "./test_data/img.png")
    });

    h1.join();
    h2.join();
}
fn main() -> anyhow::Result<()> {
    let cmd_args = CmdArgs::parse();

    let config = LedMatrixConfigDto::try_from(cmd_args.config.as_path())?;


    for _ in 0..1000 {
        let start = std::time::Instant::now();
        q();
        let elapsed = start.elapsed();
        println!("{}", elapsed.as_millis());
    }

    Ok(())
}
