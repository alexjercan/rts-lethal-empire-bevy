#![feature(test)]

extern crate test;

use bevy::prelude::*;

pub mod assets;
pub mod sampling;
pub mod states;
pub mod terrain;
pub mod building;

#[derive(Resource, Default, Debug)]
pub enum ToolMode {
    #[default]
    Select,
    Build,
}

#[derive(Component)]
pub struct Obstacle;
