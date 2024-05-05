#![feature(test)]

extern crate test;

use bevy::prelude::*;

pub mod assets;
pub mod building;
pub mod helpers;
pub mod states;
pub mod terrain;

#[cfg(feature = "debug")]
pub mod debug;

#[derive(Resource, Default, Debug)]
pub enum ToolMode {
    #[default]
    Select,
    Build,
}

#[derive(Component)]
pub struct Obstacle;
