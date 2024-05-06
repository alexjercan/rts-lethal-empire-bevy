#![feature(test)]

extern crate test;

pub mod core;
pub(crate) mod building;
pub(crate) mod helpers;
pub(crate) mod terrain;
pub(crate) mod camera;
pub(crate) mod ui;
pub(crate) mod quota;
pub(crate) mod units;

#[cfg(feature = "debug")]
pub(crate) mod debug;
