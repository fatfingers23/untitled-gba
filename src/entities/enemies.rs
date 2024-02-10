use crate::entities::entity::Entity;
use crate::level::Level;
use crate::player::{Player, PlayerAction};
use crate::types::FixedNumberType;
use agb::display::object::Graphics;
use agb::{
    display::object::{OamManaged, Tag},
    fixnum::Vector2D,
    println,
};

pub const BOAR_IDLE: &Graphics = agb::include_aseprite!("gfx/boar/Idle.aseprite");
pub const BOAR_IDLE_ANIMATION: &Tag = BOAR_IDLE.tags().get("idle");

pub const BOAR_HIT: &Graphics = agb::include_aseprite!("gfx/boar/Hit.aseprite");
pub const BOAR_HIT_ANIMATION: &Tag = BOAR_HIT.tags().get("hit");

pub const BOAR_RUN: &Graphics = agb::include_aseprite!("gfx/boar/Run.aseprite");
pub const BOAR_RUN_ANIMATION: &Tag = BOAR_RUN.tags().get("run");

// const SLIME_IDLE: &Tag = TAG_MAP.get("Slime Idle");
// const SLIME_JUMP: &Tag = TAG_MAP.get("Slime Jump");
// const SLIME_SPLAT: &Tag = TAG_MAP.get("Slime splat");
//
// const SNAIL_EMERGE: &Tag = TAG_MAP.get("Snail Emerge");
// const SNAIL_MOVE: &Tag = TAG_MAP.get("Snail Move");
// const SNAIL_DEATH: &Tag = TAG_MAP.get("Snail Death");
// const SNAIL_IDLE: &Tag = TAG_MAP.get("Snail Idle");

enum UpdateState {
    Nothing,
    KillPlayer,
    Remove,
}

#[derive(Default)]
pub enum Enemy<'a> {
    Boar(Boar<'a>),
    #[default]
    Empty,
}

pub enum EnemyUpdateState {
    None,
    KillPlayer,
}

impl<'a> Enemy<'a> {
    pub fn new_boar(object: &'a OamManaged, start_pos: Vector2D<FixedNumberType>) -> Self {
        Enemy::Boar(Boar::new(object, start_pos + (0, 1).into()))
    }

    // pub fn collides_with_hat(&self, position: Vector2D<FixedNumberType>) -> bool {
    //     match self {
    //         Enemy::Snail(snail) => snail.collides_with(position),
    //         _ => false,
    //     }
    // }

    pub fn update(
        &mut self,
        controller: &'a OamManaged,
        level: &Level,
        player_pos: Vector2D<FixedNumberType>,
        player_action: &PlayerAction,
        timer: i32,
    ) -> EnemyUpdateState {
        let update_state = match self {
            // Enemy::Slime(slime) => slime.update(controller, level, player_pos, timer),
            // Enemy::Snail(snail) => snail.update(controller, level, player_pos, timer),
            Enemy::Boar(boar) => boar.update(controller, level, player_pos, &player_action, timer),
            Enemy::Empty => UpdateState::Nothing,
        };

        match update_state {
            UpdateState::Remove => {
                *self = Enemy::Empty;
                EnemyUpdateState::None
            }
            UpdateState::KillPlayer => EnemyUpdateState::KillPlayer,
            UpdateState::Nothing => EnemyUpdateState::None,
        }
    }

    pub fn commit(&mut self, background_offset: Vector2D<FixedNumberType>) {
        match self {
            Enemy::Boar(boar) => boar.commit(background_offset),
            Enemy::Empty => {}
        }
    }
}

struct EnemyInfo<'a> {
    entity: Entity<'a>,
}

impl<'a> EnemyInfo<'a> {
    fn new(
        object: &'a OamManaged,
        start_pos: Vector2D<FixedNumberType>,
        collision: Vector2D<u16>,
    ) -> Self {
        let mut enemy_info = EnemyInfo {
            entity: Entity::new(object, collision),
        };
        enemy_info.entity.position = start_pos;
        enemy_info
    }

    fn update(&mut self, level: &Level) {
        for &enemy_stop in level.enemy_stops {
            if (self.entity.position + self.entity.velocity - enemy_stop.into())
                .manhattan_distance()
                < 8.into()
            {
                self.entity.velocity = (0, 0).into();
            }
        }
        // println!("Enemy Velocity: {:?}", self.entity.velocity);
        // self.entity.position = self.entity.position + self.entity.velocity;
        self.entity.update_position(level);
    }

    fn commit(&mut self, background_offset: Vector2D<FixedNumberType>) {
        self.entity.commit_position(background_offset);
    }
}

enum BoarState {
    Idle,
    Running(i32), // the start frame of the jumping animation
    Dying(i32),   // the start frame of the dying animation
}

pub struct Boar<'a> {
    enemy_info: EnemyInfo<'a>,
    state: BoarState,
}

impl<'a> Boar<'a> {
    fn new(object: &'a OamManaged, start_pos: Vector2D<FixedNumberType>) -> Self {
        let Boar = Boar {
            enemy_info: EnemyInfo::new(object, start_pos, (28u16, 14u16).into()),
            state: BoarState::Idle,
        };

        Boar
    }

    fn update(
        &mut self,
        controller: &'a OamManaged,
        level: &Level,
        player_pos: Vector2D<FixedNumberType>,
        player_action: &PlayerAction,
        timer: i32,
    ) -> UpdateState {
        let player_has_collided =
            (self.enemy_info.entity.position - player_pos).magnitude_squared() < (15 * 15).into();

        match self.state {
            BoarState::Idle => {
                let offset = (timer / 16) as usize;

                let frame = BOAR_IDLE_ANIMATION.animation_sprite(offset);
                let sprite = controller.sprite(frame);

                self.enemy_info.entity.sprite.set_sprite(sprite);

                if (self.enemy_info.entity.position - player_pos).magnitude_squared()
                    < (64 * 64).into()
                {
                    self.state = BoarState::Running(timer);

                    let x_vel: FixedNumberType =
                        if self.enemy_info.entity.position.x > player_pos.x {
                            -1
                        } else {
                            1
                        }
                        .into();

                    self.enemy_info.entity.velocity = (x_vel / 4, 0.into()).into();
                }

                if player_has_collided {
                    if *player_action == PlayerAction::Attack {
                        self.state = BoarState::Dying(timer);
                    } else {
                        return UpdateState::KillPlayer;
                    }
                }
            }
            BoarState::Running(jumping_start_frame) => {
                let offset = (timer - jumping_start_frame) as usize / 4;

                if timer == jumping_start_frame + 1 {
                    // Slim jumping
                }

                if offset >= 7 {
                    self.enemy_info.entity.velocity = (0, 0).into();
                    self.state = BoarState::Idle;
                } else {
                    let frame = BOAR_RUN_ANIMATION.animation_sprite(offset);
                    let sprite = controller.sprite(frame);
                    self.enemy_info.entity.sprite.set_sprite(sprite);
                }

                if player_has_collided {
                    if *player_action == PlayerAction::Attack {
                        self.state = BoarState::Dying(timer);
                    } else {
                        return UpdateState::KillPlayer;
                    }
                }
            }
            BoarState::Dying(dying_start_frame) => {
                if timer == dying_start_frame + 1 {
                    // sfx_player.Boar_death();
                }

                let offset = (timer - dying_start_frame) as usize / 4;
                self.enemy_info.entity.velocity = (0, 0).into();

                if offset >= 4 {
                    return UpdateState::Remove;
                }

                let frame = BOAR_HIT_ANIMATION.animation_sprite(offset);
                let sprite = controller.sprite(frame);

                self.enemy_info.entity.sprite.set_sprite(sprite);
            }
        }

        self.enemy_info.update(level);

        UpdateState::Nothing
    }

    fn commit(&mut self, background_offset: Vector2D<FixedNumberType>) {
        self.enemy_info.commit(background_offset);
    }
}
