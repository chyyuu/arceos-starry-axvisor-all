#![cfg_attr(not(feature = "std"), no_std)]
#![feature(let_chains)]

use axvmconfig::*;

#[cfg(feature = "std")]
mod tool;

#[cfg(feature = "std")]
mod templates;

fn main() {
    // configure logger and set log level
    #[cfg(feature = "std")]
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Debug)
        .init();
    #[cfg(feature = "std")]
    tool::run();
}
