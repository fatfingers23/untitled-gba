use crate::display_text;
use agb::display::tiled::{RegularMap, TiledMap, VRamManager};
use alloc::format;

agb::include_background_gfx!(splash_screens,
    splash => deduplicate "gfx/Background.png",
);

pub enum SplashScreen {
    Start,
    End,
}

pub fn show_splash_screen(which: SplashScreen, map: &mut RegularMap, vram: &mut VRamManager) {
    map.set_scroll_pos((0i16, 0i16).into());
    let tile_data = match which {
        SplashScreen::Start => splash_screens::splash,
        SplashScreen::End => splash_screens::splash,
    };

    let vblank = agb::interrupt::VBlank::get();

    let mut input = agb::input::ButtonController::new();

    vblank.wait_for_vblank();

    map.fill_with(vram, &tile_data);
    // display_text::write(map, format!("Press Start to begin"), vram);

    map.commit(vram);
    vram.set_background_palettes(splash_screens::PALETTES);
    map.show();

    loop {
        input.update();
        if input.is_just_pressed(
            agb::input::Button::A
                | agb::input::Button::B
                | agb::input::Button::START
                | agb::input::Button::SELECT,
        ) {
            break;
        }

        vblank.wait_for_vblank();
    }

    map.hide();
    map.clear(vram);
}
