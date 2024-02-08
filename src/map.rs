use crate::level::Level;
use crate::types::FixedNumberType;
use agb::display::tiled::{InfiniteScrolledMap, PartialUpdateStatus, VRamManager};
use agb::fixnum::Vector2D;

pub struct Map<'a, 'b> {
    pub background: &'a mut InfiniteScrolledMap<'b>,
    pub foreground: &'a mut InfiniteScrolledMap<'b>,
    pub position: Vector2D<FixedNumberType>,
    pub level: &'a Level<'a>,
}

impl<'a, 'b> Map<'a, 'b> {
    pub fn commit_position(&mut self, vram: &mut VRamManager) {
        self.background.set_pos(vram, self.position.floor());
        self.foreground.set_pos(vram, self.position.floor());

        self.background.commit(vram);
        self.foreground.commit(vram);
    }

    pub fn init_background(&mut self, vram: &mut VRamManager) -> PartialUpdateStatus {
        self.background.init_partial(vram, self.position.floor())
    }

    pub fn init_foreground(&mut self, vram: &mut VRamManager) -> PartialUpdateStatus {
        self.foreground.init_partial(vram, self.position.floor())
    }
}
