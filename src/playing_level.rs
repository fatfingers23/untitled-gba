use crate::entities::enemies;
use crate::level::Level;
use crate::map::Map;
use crate::player::{Player, WARRIOR_DEAD_END_ANIMATION, WARRIOR_DEAD_START_ANIMATION};
use crate::types::{FixedNumberType, TILE_SIZE};
use agb::display::object::{Graphics, OamManaged, Tag};
use agb::display::tiled::{InfiniteScrolledMap, VRamManager};
use agb::display::{Priority, HEIGHT, WIDTH};
use agb::fixnum::Vector2D;
use agb::input::ButtonController;
use UpdateState::{Complete, Dead, Normal};

pub struct PlayingLevel<'a, 'b> {
    pub timer: i32,
    pub background: Map<'a, 'b>,
    pub input: ButtonController,
    pub player: Player<'a>,
    enemies: [enemies::Enemy<'a>; 16],
}

pub enum UpdateState {
    Normal,
    Dead,
    Complete,
}

impl<'a, 'b> PlayingLevel<'a, 'b> {
    pub fn open_level(
        level: &'a Level,
        object_control: &'a OamManaged,
        background: &'a mut InfiniteScrolledMap<'b>,
        foreground: &'a mut InfiniteScrolledMap<'b>,
        input: ButtonController,
    ) -> Self {
        let mut e: [enemies::Enemy<'a>; 16] = Default::default();
        let mut enemy_count = 0;
        for &boar in level.boars {
            e[enemy_count] = enemies::Enemy::new_boar(object_control, boar.into());
            enemy_count += 1;
        }

        let start_pos: Vector2D<FixedNumberType> = level.start_pos.into();

        let background_position = (
            (start_pos.x - WIDTH / 2).clamp(
                0.into(),
                ((level.dimensions.x * TILE_SIZE) as i32 - WIDTH).into(),
            ),
            (start_pos.y - HEIGHT / 2).clamp(
                0.into(),
                ((level.dimensions.y * TILE_SIZE) as i32 - HEIGHT).into(),
            ),
        )
            .into();

        PlayingLevel {
            timer: 0,
            background: Map {
                background,
                foreground,
                level,
                position: background_position,
            },
            player: Player::new(object_control, start_pos),
            input,
            enemies: e,
        }
    }

    pub fn show_backgrounds(&mut self) {
        self.background.background.show();
        self.background.foreground.show();
    }

    pub fn hide_backgrounds(&mut self) {
        self.background.background.hide();
        self.background.foreground.hide();
    }

    pub fn clear_backgrounds(&mut self, vram: &mut VRamManager) {
        self.background.background.clear(vram);
        self.background.foreground.clear(vram);
    }

    pub fn dead_start(&mut self) {
        self.player.warrior.velocity = (0, -1).into();
        self.player.warrior.sprite.set_priority(Priority::P0);
    }

    /// Controls mostly the dead animation while *dying*
    pub fn dead_update(&mut self, controller: &'a OamManaged, animation_frame: usize) {
        self.timer += 1;
        if animation_frame <= 4 {
            let start_frame = WARRIOR_DEAD_START_ANIMATION.animation_sprite(animation_frame);
            self.player
                .warrior
                .sprite
                .set_sprite(controller.sprite(start_frame));
            self.player
                .warrior
                .commit_position(self.background.position);
        } else {
            let frame = WARRIOR_DEAD_END_ANIMATION.animation_sprite(animation_frame);
            self.player
                .warrior
                .sprite
                .set_sprite(controller.sprite(frame));
            self.player
                .warrior
                .commit_position(self.background.position - (0, 15).into());
        }
    }

    pub fn update_frame(
        &mut self,
        vram: &mut VRamManager,
        controller: &'a OamManaged,
    ) -> UpdateState {
        self.timer += 1;
        self.input.update();

        let mut player_dead = false;
        self.player
            .update_frame(&self.input, controller, self.timer, self.background.level);

        for enemy in self.enemies.iter_mut() {
            match enemy.update(
                controller,
                self.background.level,
                self.player.warrior.position,
                &self.player.action,
                self.timer,
            ) {
                enemies::EnemyUpdateState::KillPlayer => player_dead = true,
                enemies::EnemyUpdateState::None => {}
            }
        }

        self.background.position = self.get_next_map_position();
        self.background.commit_position(vram);

        self.player
            .warrior
            .commit_position(self.background.position - self.player.sprite_off_set);

        // self.player.hat.commit_position(self.background.position);

        for enemy in self.enemies.iter_mut() {
            enemy.commit(self.background.position);
        }

        player_dead |= self
            .player
            .warrior
            .killision_at_point(self.background.level, self.player.warrior.position);
        if player_dead {
            Dead
        } else if self
            .player
            .warrior
            .completion_at_point(self.background.level, self.player.warrior.position)
        {
            Complete
        } else {
            Normal
        }
    }

    fn get_next_map_position(&self) -> Vector2D<FixedNumberType> {
        // want to ensure the player and the hat are visible if possible, so try to position the map
        // so the centre is at the average position. But give the player some extra priority

        let player_pos = self.player.warrior.position.floor();
        let new_target_position = (player_pos * 3) / 4;

        let screen: Vector2D<i32> = (WIDTH, HEIGHT).into();
        let half_screen = screen / 2;
        let current_centre = self.background.position.floor() + half_screen;

        let mut target_position = ((current_centre * 3 + new_target_position) / 4) - half_screen;

        target_position.x = target_position.x.clamp(
            0,
            (self.background.level.dimensions.x * TILE_SIZE - (WIDTH as u32)) as i32,
        );
        target_position.y = target_position.y.clamp(
            0,
            (self.background.level.dimensions.y * TILE_SIZE - (HEIGHT as u32)) as i32,
        );

        target_position.into()
    }
}
