#![no_std]
#![no_main]
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]
#![cfg_attr(test, test_runner(agb::test_runner::test_runner))]

mod display_text;
mod entities;
mod level;
mod map;
mod player;
mod playing_level;
mod splash_screen;
mod sprites;
mod types;

extern crate alloc;

use crate::level::map_tiles;
use crate::playing_level::{PlayingLevel, UpdateState};
use agb::display::tiled::{
    InfiniteScrolledMap, PartialUpdateStatus, RegularBackgroundSize, TileFormat, TiledMap,
};
use agb::display::{Font, Priority};
use agb::fixnum::Vector2D;
use agb::include_font;
use alloc::boxed::Box;
use alloc::format;

const LEVEL_LOADING_SCREEN_WAIT: i32 = 5;

agb::include_background_gfx!(
    games, "2ce8f4",
    level_1_background  => 16 deduplicate "gfx/tileSets/level_1/level_1_background.png",
    level_1_foreground => 16  deduplicate "gfx/tileSets/level_1/level_1_foreground.png"
);
const FONT_14: Font = include_font!("font/pixelated.ttf", 14);

pub fn main(mut agb: agb::Gba) -> ! {
    let (tiled, mut vram) = agb.display.video.tiled0();
    vram.set_background_palettes(games::PALETTES);
    let mut _splash_screen = tiled.background(
        Priority::P0,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );
    let mut world_display = tiled.background(
        Priority::P0,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );

    // let tileset = games::level_1_background.tiles;

    // for y in 0..32u16 {
    //     for x in 0..32u16 {
    //         world_display.set_tile(
    //             &mut vram,
    //             (x, y).into(),
    //             &tileset,
    //             games::level_1_background.tile_settings[level_display::BLANK],
    //         );
    //     }
    // }

    world_display.commit(&mut vram);
    world_display.show();

    // splash_screen::show_splash_screen(
    //     splash_screen::SplashScreen::Start,
    //     &mut splash_screen,
    //     &mut vram,
    // );

    loop {
        world_display.commit(&mut vram);
        world_display.show();

        let object = agb.display.object.get_managed();

        let vblank = agb::interrupt::VBlank::get();
        let mut current_level = 0;

        loop {
            if current_level == map_tiles::LEVELS.len() as u32 {
                break;
            }

            vblank.wait_for_vblank();

            let current_level_text = format!("level {}", current_level + 1);
            display_text::write(&mut world_display, current_level_text, &mut vram);

            world_display.commit(&mut vram);
            world_display.show();

            vblank.wait_for_vblank();

            let map_current_level = current_level;

            vram.set_background_palettes(games::PALETTES);
            let mut background = InfiniteScrolledMap::new(
                tiled.background(
                    Priority::P0,
                    RegularBackgroundSize::Background32x64,
                    TileFormat::FourBpp,
                ),
                Box::new(|pos: Vector2D<i32>| {
                    let level = &map_tiles::LEVELS[map_current_level as usize];
                    let index = tile_index_math(pos.y, pos.x, level.dimensions.x as i32);

                    (
                        &level.background_tile_set,
                        level.background_tile_settings
                            [*level.background.get(index).unwrap_or(&0) as usize],
                    )
                }),
            );
            let mut foreground = InfiniteScrolledMap::new(
                tiled.background(
                    Priority::P2,
                    RegularBackgroundSize::Background64x32,
                    TileFormat::FourBpp,
                ),
                Box::new(|pos: Vector2D<i32>| {
                    let level = &map_tiles::LEVELS[map_current_level as usize];
                    let index = tile_index_math(pos.y, pos.x, level.dimensions.x as i32);
                    let tile_file_index = *level.foreground.get(index).unwrap_or(&0) as usize;

                    (
                        &level.foreground_tile_set,
                        level.foreground_tile_settings[tile_file_index],
                    )
                }),
            );

            let mut level = PlayingLevel::open_level(
                &map_tiles::LEVELS[current_level as usize],
                &object,
                &mut background,
                &mut foreground,
                agb::input::ButtonController::new(),
            );

            while level.background.init_background(&mut vram) != PartialUpdateStatus::Done {
                vblank.wait_for_vblank();
            }

            while level.background.init_foreground(&mut vram) != PartialUpdateStatus::Done {
                vblank.wait_for_vblank();
            }

            for _ in 0..LEVEL_LOADING_SCREEN_WAIT {
                vblank.wait_for_vblank();
            }

            object.commit();

            level.show_backgrounds();

            world_display.hide();

            loop {
                match level.update_frame(&mut vram, &object) {
                    UpdateState::Normal => {}
                    UpdateState::Dead => {
                        // display_text::write(
                        //     &mut world_display,
                        //     format!("That did not go as planned"),
                        //     &mut vram,
                        // );

                        level.dead_start();

                        for i in 0..=8 {
                            level.dead_update(&object, i);
                            object.commit();
                            if i != 5 {
                                delay(&vblank, 7);
                            }
                        }

                        break;
                    }
                    UpdateState::Complete => {
                        current_level += 1;
                        break;
                    }
                }

                // sfx.frame();
                vblank.wait_for_vblank();
                object.commit();
            }

            level.hide_backgrounds();
            level.clear_backgrounds(&mut vram);
        }

        object.commit();

        // splash_screen::show_splash_screen(
        //     splash_screen::SplashScreen::End,
        //     &mut sfx,
        //     &mut splash_screen,
        //     &mut vram,
        // );
    }

    fn tile_index_math(y: i32, x: i32, width: i32) -> usize {
        (y * width as i32 + x) as usize
    }

    fn delay(vblank: &agb::interrupt::VBlank, frames: u32) {
        for _ in 0..frames {
            vblank.wait_for_vblank();
        }
    }
}
