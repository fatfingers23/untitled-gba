use agb::display::tiled::{TileSet, TileSetting};
use agb::fixnum::Vector2D;
use agb::println;

pub mod map_tiles {
    use super::Level;
    pub const LEVELS: &[Level] = &[level_one::get_level(), level_two::get_level()];

    pub mod level_one {
        include!(concat!(env!("OUT_DIR"), "/level_1.rs"));
    }

    pub mod level_two {
        include!(concat!(env!("OUT_DIR"), "/level_2.rs"));
    }

    pub mod tilemap {
        pub const COLLISION_TILE: i32 = 1;
        pub const KILL_TILE: i32 = 2;
        pub const WIN_TILE: i32 = 4;
    }
}

pub struct Level<'a> {
    pub background: &'static [u16],
    pub foreground: &'static [u16],
    pub dimensions: Vector2D<u32>,
    pub background_collision: &'static [u32],
    pub foreground_collision: &'static [u32],
    pub slimes: &'static [(i32, i32)],
    pub boars: &'static [(i32, i32)],
    pub snails: &'static [(i32, i32)],
    pub enemy_stops: &'static [(i32, i32)],
    pub start_pos: (i32, i32),
    pub background_tile_set: TileSet<'a>,
    pub background_tile_settings: &'static [TileSetting],
    pub foreground_tile_set: TileSet<'a>,
    pub foreground_tile_settings: &'static [TileSetting],
}

impl<'a> Level<'a> {
    pub fn collides(&self, x: i32, y: i32) -> bool {
        self.at_point(x, y, map_tiles::tilemap::COLLISION_TILE as u32)
    }

    pub fn kills(&self, x: i32, y: i32) -> bool {
        self.at_point(x, y, map_tiles::tilemap::KILL_TILE as u32)
    }

    pub fn at_point(&self, x: i32, y: i32, tile: u32) -> bool {
        if (x < 0 || x >= self.dimensions.x as i32) || (y < 0 || y >= self.dimensions.y as i32) {
            return true;
        }
        let pos = (self.dimensions.x as i32 * y + x) as usize;
        let tile_foreground = self.foreground[pos];
        let tile_background = self.background[pos];

        let foreground_tile_property = self.foreground_collision[tile_foreground as usize];

        let mut background_collision = false;
        if tile_background <= self.background_collision.len() as u16 {
            let background_tile_property = self.background_collision[tile_background as usize];
            background_collision = background_tile_property == tile;
        }
        let foreground_collision = foreground_tile_property == tile;

        if background_collision {
            println!("x: {}, y: {}", x, y);
            println!("Background collision  ({}, {})", tile_background, tile);
        }
        //
        // if foreground_collision {
        //     println!("Foreground collision  ({}, {})", tile_foreground, tile);
        // }

        foreground_collision || background_collision
    }

    pub fn wins(&self, x: i32, y: i32) -> bool {
        self.at_point(x, y, map_tiles::tilemap::WIN_TILE as u32)
    }
}
