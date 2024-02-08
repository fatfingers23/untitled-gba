use agb::display::object::{Graphics, Tag};

pub const WARRIOR_IDLE: &Graphics = agb::include_aseprite!("gfx/warrior/Idle.aseprite");
pub const WARRIOR_IDLE_ANIMATION: &Tag = WARRIOR_IDLE.tags().get("idle");
pub const WARRIOR_RUN: &Graphics = agb::include_aseprite!("gfx/warrior/Run.aseprite");
pub const WARRIOR_RUN_ANIMATION: &Tag = WARRIOR_RUN.tags().get("running");
pub const WARRIOR_JUMP: &Graphics = agb::include_aseprite!("gfx/warrior/Jump.aseprite");
pub const WARRIOR_JUMP_ANIMATION: &Tag = WARRIOR_JUMP.tags().get("Loop");
