use crate::types::TILE_SIZE;
use crate::FONT_14;
use agb::display::{
    tiled::{RegularMap, TileSet, TileSetting, VRamManager},
    HEIGHT, WIDTH,
};
use agb::fixnum::Vector2D;
use alloc::string::String;
use core::fmt::Write;

pub const BLANK: usize = 704;

pub fn write(map: &mut RegularMap, text: String, vram: &mut VRamManager) {
    let mut text_renderer = FONT_14.render_text(Vector2D::new(65, 0));
    let mut text_writer = text_renderer.writer(8, 0, map, vram);

    writeln!(&mut text_writer, "{text}").unwrap();
    text_writer.commit();

    map.set_scroll_pos(
        (
            -(WIDTH / 2 - 7 * TILE_SIZE as i32 / 2) as i16,
            -(HEIGHT / 2 - 4) as i16,
        )
            .into(),
    );
    text_renderer.clear(vram);
}
