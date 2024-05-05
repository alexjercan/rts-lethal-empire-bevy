use bevy::prelude::*;
use lethal_empire_bevy::core::LethalEmpirePlugin;

// # Version 0.2
// - [x] Tile based map V2
//   - [x] create a system that can extend the map in any direction
//   - [x] implement loading and unloading tiles when scrolling trough the map
//   - [x] keep only 3x3 tilemaps around camera
//   - [x] keep the rest of the chunks loaded and updated but not shown
// - [x] Resources
//   - [x] use TileKind as the base of the tile: e.g Water, Land, BarrenLand
//   - [x] implement Poisson disc distribution for nicer resource patches in a tile
//   - [x] implement additional noise layer that will be used for each resource type
//   - [x] do actually nicer resource patches
// - [x] fix the seed (again...): use a rng to generate seeds
// - [x] Refactor
//   - [x] PDD do it with builder pattern
// - [ ] Buildings
//   - [x] placing building on the map
//   - [x] models for buildings (really basic ones, I only need them to be there as a proof of concept)
//   - [ ] with units that can get resources from the map (really basic they can go trough things)
// - [ ] Pathfinding
//   - [ ] units that can move on the map based on tiles
// - [ ] Main Goal
//   - [ ] need to pay quota of resources to the Empire over time
//   - [ ] UI with the timer and quota needed and also how much we have
//   - "TIME LEFT: 10:00" "QUOTA: 500/1000"
// - [ ] Better Camera: Using PanOrbitCameraPlugin for now, but we will need to create our own camera
// - [ ] plan for V3

fn main() {
    App::new().add_plugins(LethalEmpirePlugin).run();
}
