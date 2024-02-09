use crate::entities::entity::Entity;
use crate::level::Level;
use crate::player::PlayerAction::{DoubleJump, Idle};
use crate::types::FixedNumberType;
use agb::display::object::{Graphics, OamManaged, Tag};
use agb::fixnum::Vector2D;
use agb::input::{Button, ButtonController};
use agb::{input, println};

const X_VELOCITY: i32 = 2;

pub const WARRIOR_IDLE: &Graphics = agb::include_aseprite!("gfx/warrior/Idle.aseprite");
pub const WARRIOR_IDLE_ANIMATION: &Tag = WARRIOR_IDLE.tags().get("idle");

pub const WARRIOR_RUN: &Graphics = agb::include_aseprite!("gfx/warrior/Run.aseprite");
pub const WARRIOR_RUN_ANIMATION: &Tag = WARRIOR_RUN.tags().get("running");

pub const WARRIOR_JUMP: &Graphics = agb::include_aseprite!("gfx/warrior/Jump.aseprite");
pub const WARRIOR_JUMP_ANIMATION: &Tag = WARRIOR_JUMP.tags().get("Loop");

pub const WARRIOR_DEAD_START: &Graphics = agb::include_aseprite!("gfx/warrior/DeadStart.aseprite");
pub const WARRIOR_DEAD_START_ANIMATION: &Tag = WARRIOR_DEAD_START.tags().get("dead");

pub const WARRIOR_DEAD_END: &Graphics = agb::include_aseprite!("gfx/warrior/DeadEnd.aseprite");
pub const WARRIOR_DEAD_END_ANIMATION: &Tag = WARRIOR_DEAD_END.tags().get("dead");

pub const WARRIOR_ATTACK: &Graphics = agb::include_aseprite!("gfx/warrior/NewAttack.aseprite");
pub const WARRIOR_ATTACK_ANIMATION: &Tag = WARRIOR_ATTACK.tags().get("attack");

#[derive(Debug, PartialEq)]
pub enum PlayerAction {
    Idle,
    Run,
    Dash,
    Jump,
    DoubleJump,
    Attack,
}

pub struct Player<'a> {
    pub warrior: Entity<'a>,
    pub hat_left_range: bool,
    pub hat_slow_counter: i32,
    pub warrior_frame: u8,
    pub num_recalls: i8,
    pub is_on_ground: bool,
    pub facing: input::Tri,
    pub last_idle_frame: i32,
    pub times_last_attack_frame_displayed: i32,
    /// Some sprites do not line up perfectly. Attack for instance does not
    /// this lets you control it a bit without changing the Player's position
    pub sprite_off_set: Vector2D<FixedNumberType>,
    pub action: PlayerAction,
}

impl<'a> Player<'a> {
    pub fn new(controller: &'a OamManaged, start_position: Vector2D<FixedNumberType>) -> Self {
        let mut warrior = Entity::new(controller, (16_u16, 16_u16).into());
        //
        warrior
            .sprite
            .set_sprite(controller.sprite(WARRIOR_IDLE.sprites().first().unwrap()));
        // warrior.position = start_position + (0, -7).into();
        warrior.position = (0, 100).into();

        Player {
            warrior,
            hat_slow_counter: 0,
            hat_left_range: false,
            warrior_frame: 0,
            num_recalls: 0,
            is_on_ground: true,
            facing: input::Tri::Zero,
            last_idle_frame: 0,
            times_last_attack_frame_displayed: 0,
            sprite_off_set: (0, 0).into(),
            action: PlayerAction::Idle,
        }
    }

    pub fn update_frame(
        &mut self,
        input: &ButtonController,
        controller: &'a OamManaged,
        timer: i32,
        level: &Level,
    ) {
        //TODO not sure how to do a double dash. Maybe count a few 0s before -1 or 1
        //OR could just do longer. GUess i could do like holding down r or attack?
        let x = input.x_tri() as i32;
        println!("Input x {x}");

        //     // throw or recall
        //     if input.is_just_pressed(Button::A) {
        //         if self.hat_state == HatState::OnHead {
        //             let direction: Vector2D<FixedNumberType> = {
        //                 let up_down = input.y_tri() as i32;
        //                 let left_right = if up_down == 0 {
        //                     self.facing as i32
        //                 } else {
        //                     input.x_tri() as i32
        //                 };]
        //                 (left_right, up_down).into()
        //             };
        //
        //             if direction != (0, 0).into() {
        //                 let mut velocity = direction.normalise() * 5;
        //                 if velocity.y > 0.into() {
        //                     velocity.y *= FixedNumberType::new(4) / 3;
        //                 }
        //                 self.hat.velocity = velocity;
        //                 self.hat_state = HatState::Thrown;
        //
        //                 sfx_player.throw();
        //             }
        //         } else if self.hat_state == HatState::Thrown {
        //             self.num_recalls += 1;
        //             if self.num_recalls < 3 {
        //                 self.hat.velocity = (0, 0).into();
        //                 self.warrior.velocity = (0, 0).into();
        //                 self.hat_state = HatState::warriorTowards;
        //             }
        //         } else if self.hat_state == HatState::warriorTowards {
        //             self.hat_state = HatState::Thrown;
        //             self.warrior.velocity /= 8;
        //         }
        //     }
        //
        let was_on_ground = self.is_on_ground;
        let is_on_ground = self
            .warrior
            .collision_at_point(level, self.warrior.position + (0, 1).into());
        if is_on_ground && !was_on_ground && self.warrior.velocity.y > 1.into() {
            self.action = PlayerAction::Idle;
        }
        self.is_on_ground = is_on_ground;

        //On the ground
        if is_on_ground {
            self.warrior.velocity.x += FixedNumberType::new(input.x_tri() as i32 * X_VELOCITY) / 16;
            self.warrior.velocity = self.warrior.velocity * 54 / 64;
            // if self.action != PlayerAction::Attack {
            //     self.warrior.velocity.x +=
            //         FixedNumberType::new(input.x_tri() as i32 * X_VELOCITY) / 16;
            //     self.warrior.velocity = self.warrior.velocity * 54 / 64;
            // } else {
            //     self.warrior.velocity = (0, 0).into();
            // }
            //Jump
            if input.is_just_pressed(Button::A) {
                self.warrior.velocity.y = -FixedNumberType::new(3) / 2;
                self.action = PlayerAction::Jump;
            }
        } else {
            //Double jump
            if self.action != DoubleJump {
                if input.is_just_pressed(Button::A) {
                    self.warrior.velocity.y = -FixedNumberType::new(3) / 2;
                    self.action = DoubleJump;
                }
            }
            self.warrior.velocity.x += FixedNumberType::new(input.x_tri() as i32) / 64;
            self.warrior.velocity = self.warrior.velocity * 63 / 64;
            let gravity: Vector2D<FixedNumberType> = (0, 1).into();
            let gravity = gravity / 16;
            self.warrior.velocity += gravity;
        }
        self.warrior.velocity = self.warrior.update_position(level);

        //Running
        if self.warrior.velocity.x.abs() > 0.into() {
            let offset = (ping_pong(timer / 16, 4)) as usize;
            self.warrior_frame = offset as u8;
            let frame = WARRIOR_RUN_ANIMATION.animation_sprite(offset);
            let sprite = controller.sprite(frame);
            if self.action != PlayerAction::Attack {
                self.warrior.sprite.set_sprite(sprite);
            }
            if self.action == PlayerAction::Idle {
                self.action = PlayerAction::Run;
            }
        } else {
            if self.action != PlayerAction::Attack {
                self.action = PlayerAction::Idle;
            }
        }

        // Set logic of jump sprite
        if self.warrior.velocity.y < -FixedNumberType::new(1) / 16 {
            // going up
            self.warrior_frame = 5;
            let offset = (timer / 16) as usize;
            let frame = WARRIOR_JUMP_ANIMATION.animation_sprite(offset);
            let sprite = controller.sprite(frame);
            if self.action != PlayerAction::Attack {
                self.warrior.sprite.set_sprite(sprite);
            }
        } else if self.warrior.velocity.y > FixedNumberType::new(1) / 16 {
            // going down
            let _offset = if self.warrior.velocity.y * 2 > 3.into() {
                (timer / 4) as usize
            } else {
                // Don't flap beard unless going quickly
                0
            };

            self.warrior_frame = 0;
            // Can set the sprite for falling here if we want one for up or down
        }

        if input.x_tri() != agb::input::Tri::Zero {
            self.facing = input.x_tri();
        }

        match self.facing {
            agb::input::Tri::Negative => {
                self.warrior.sprite.set_hflip(true);
            }
            agb::input::Tri::Positive => {
                self.warrior.sprite.set_hflip(false);
            }
            _ => {}
        }

        //Attack
        if input.is_just_pressed(Button::B) && self.is_on_ground {
            if self.action != PlayerAction::Attack {
                if self.facing == agb::input::Tri::Positive {
                    self.sprite_off_set = (-16, 0).into();
                    // self.warrior.position = self.warrior.position - (16, 0).into();
                }
            }
            self.action = PlayerAction::Attack;
        }

        if self.action == PlayerAction::Attack {
            let offset = (timer / 16) as usize;
            let animation_length = WARRIOR_ATTACK_ANIMATION.sprites().len();
            let animation_frame = offset % animation_length;

            if animation_frame + 1 >= animation_length {
                self.times_last_attack_frame_displayed += 1;

                //Delay to show last frame a few extra times
                if self.times_last_attack_frame_displayed > 2 {
                    self.action = PlayerAction::Idle;
                    self.times_last_attack_frame_displayed = 0;
                    if self.facing == agb::input::Tri::Positive {
                        self.sprite_off_set = (0, 0).into();
                    }
                }
            }
            self.new_attack_frame(controller, animation_frame);
        }

        if self.action == Idle && self.is_on_ground {
            self.sprite_off_set = (0, 0).into();
            self.action = PlayerAction::Idle;
            self.new_idle_frame(controller, timer);
        }
    }

    fn new_idle_frame(&mut self, controller: &'a OamManaged, timer: i32) {
        let offset = (timer / 32) as usize;
        let frame = WARRIOR_IDLE_ANIMATION.animation_sprite(offset);
        let sprite = controller.sprite(frame);
        self.warrior.sprite.set_sprite(sprite);
    }

    fn new_attack_frame(&mut self, controller: &'a OamManaged, frame_number: usize) {
        let frame = WARRIOR_ATTACK_ANIMATION.animation_sprite(frame_number);
        let sprite = controller.sprite(frame);
        self.warrior.sprite.set_sprite(sprite);
    }
}

fn ping_pong(i: i32, n: i32) -> i32 {
    let cycle = 2 * (n - 1);
    let i = i % cycle;
    if i >= n {
        cycle - i
    } else {
        i
    }
}
