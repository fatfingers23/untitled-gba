use crate::entities::entity::Entity;
use crate::level::Level;
use crate::sprites::{
    WARRIOR_ATTACK_ANIMATION, WARRIOR_IDLE, WARRIOR_IDLE_ANIMATION, WARRIOR_JUMP_ANIMATION,
    WARRIOR_RUN_ANIMATION,
};
use crate::types::FixedNumberType;
use agb::display::object::OamManaged;
use agb::fixnum::Vector2D;
use agb::input::{Button, ButtonController};
use agb::{input, println};

const X_VELOCITY: i32 = 2;
pub struct Player<'a> {
    pub warrior: Entity<'a>,
    pub hat_left_range: bool,
    pub hat_slow_counter: i32,
    pub warrior_frame: u8,
    pub num_recalls: i8,
    pub is_on_ground: bool,
    pub facing: input::Tri,
    pub last_idle_frame: i32,
    pub has_double_jumped: bool,
    pub attacking: bool,
    pub times_last_attack_frame_displayed: i32,
}

impl<'a> Player<'a> {
    pub fn new(controller: &'a OamManaged, start_position: Vector2D<FixedNumberType>) -> Self {
        let mut warrior = Entity::new(controller, (6, 16_u16).into());
        //
        warrior
            .sprite
            .set_sprite(controller.sprite(WARRIOR_IDLE.sprites().first().unwrap()));
        warrior.position = start_position + (0, -7).into();

        Player {
            warrior,
            hat_slow_counter: 0,
            hat_left_range: false,
            warrior_frame: 0,
            num_recalls: 0,
            is_on_ground: true,
            facing: input::Tri::Zero,
            last_idle_frame: 0,
            has_double_jumped: false,
            attacking: false,
            times_last_attack_frame_displayed: 0,
        }
    }

    pub fn update_frame(
        &mut self,
        input: &ButtonController,
        controller: &'a OamManaged,
        timer: i32,
        level: &Level,
    ) {
        let mut any_movement = false;

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
            self.has_double_jumped = false;
        }
        self.is_on_ground = is_on_ground;
        //
        //     if self.hat_state != HatState::warriorTowards {
        //         if is_on_ground {
        //             self.num_recalls = 0;
        //         }
        //
        if is_on_ground {
            self.warrior.velocity.x += FixedNumberType::new(input.x_tri() as i32 * X_VELOCITY) / 16;
            self.warrior.velocity = self.warrior.velocity * 54 / 64;
            //Jump
            if input.is_just_pressed(Button::A) {
                self.warrior.velocity.y = -FixedNumberType::new(3) / 2;
                any_movement = true;
                // sfx_player.jump();
                // println!("jump")
            }
        } else {
            //Double jump
            if !self.has_double_jumped {
                if input.is_just_pressed(Button::A) {
                    self.warrior.velocity.y = -FixedNumberType::new(3) / 2;
                    self.has_double_jumped = true;
                    any_movement = true;
                }
            }
            self.warrior.velocity.x += FixedNumberType::new(input.x_tri() as i32) / 64;
            self.warrior.velocity = self.warrior.velocity * 63 / 64;
            let gravity: Vector2D<FixedNumberType> = (0, 1).into();
            let gravity = gravity / 16;
            self.warrior.velocity += gravity;
        }
        //
        self.warrior.velocity = self.warrior.update_position(level);

        if self.warrior.velocity.x.abs() > 0.into() {
            let offset = (ping_pong(timer / 16, 4)) as usize;
            self.warrior_frame = offset as u8;
            any_movement = true;
            let frame = WARRIOR_RUN_ANIMATION.animation_sprite(offset);
            let sprite = controller.sprite(frame);
            self.warrior.sprite.set_sprite(sprite);
        }
        //
        if self.warrior.velocity.y < -FixedNumberType::new(1) / 16 {
            // going up
            self.warrior_frame = 5;
            let offset = (timer / 16) as usize;
            let frame = WARRIOR_JUMP_ANIMATION.animation_sprite(offset);
            let sprite = controller.sprite(frame);

            self.warrior.sprite.set_sprite(sprite);
        } else if self.warrior.velocity.y > FixedNumberType::new(1) / 16 {
            // going down
            let offset = if self.warrior.velocity.y * 2 > 3.into() {
                (timer / 4) as usize
            } else {
                // Don't flap beard unless going quickly
                0
            };

            self.warrior_frame = 0;

            // let frame = FALLING.animation_sprite(offset);
            // let sprite = controller.sprite(frame);

            // self.warrior.sprite.set_sprite(sprite);
        }

        if input.x_tri() != agb::input::Tri::Zero {
            self.facing = input.x_tri();
        }

        //
        //     let hat_base_tile = match self.num_recalls {
        //         0 => HAT_SPIN_1,
        //         1 => HAT_SPIN_2,
        //         _ => HAT_SPIN_3,
        //     };
        //
        //     let hat_resting_position = match self.warrior_frame {
        //         1 | 2 => (0, 9).into(),
        //         5 => (0, 10).into(),
        //         _ => (0, 8).into(),
        //     };
        //

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
        if input.is_just_pressed(Button::B) {
            if !self.attacking {
                if self.facing == agb::input::Tri::Positive {
                    self.warrior.position = self.warrior.position - (15, 0).into();
                }
            }
            self.attacking = true;
        }

        if self.attacking {
            let offset = (timer / 16) as usize;
            let animation_length = WARRIOR_ATTACK_ANIMATION.sprites().len();
            let animation_frame = offset % animation_length;

            if animation_frame + 1 >= animation_length {
                self.times_last_attack_frame_displayed += 1;

                //Delay to show last frame a few extra times
                if self.times_last_attack_frame_displayed > 2 {
                    self.attacking = false;
                    self.times_last_attack_frame_displayed = 0;
                    self.warrior.position = self.warrior.position + (15, 0).into();
                }
            }
            self.new_attack_frame(controller, animation_frame);
            any_movement = true;
        }
        //
        //     match self.hat_state {
        //         HatState::Thrown => {
        //             // hat is thrown, make hat move towards warrior
        //             let distance_vector =
        //                 self.warrior.position - self.hat.position - hat_resting_position;
        //             let distance = distance_vector.magnitude();
        //             let direction = if distance == 0.into() {
        //                 (0, 0).into()
        //             } else {
        //                 distance_vector / distance
        //             };
        //
        //             let hat_sprite_divider = match self.num_recalls {
        //                 0 => 1,
        //                 1 => 2,
        //                 _ => 4,
        //             };
        //
        //             let hat_sprite_offset = (timer / hat_sprite_divider) as usize;
        //
        //             self.hat.sprite.set_sprite(
        //                 controller.sprite(hat_base_tile.animation_sprite(hat_sprite_offset)),
        //             );
        //
        //             if self.hat_slow_counter < 30 && self.hat.velocity.magnitude() < 2.into() {
        //                 self.hat.velocity = (0, 0).into();
        //                 self.hat_slow_counter += 1;
        //             } else {
        //                 self.hat.velocity += direction / 4;
        //             }
        //             let (new_velocity, enemy_collision) =
        //                 self.hat.update_position_with_enemy(level, enemies);
        //             self.hat.velocity = new_velocity;
        //
        //             if enemy_collision {
        //                 sfx_player.snail_hat_bounce();
        //             }
        //
        //             if distance > 16.into() {
        //                 self.hat_left_range = true;
        //             }
        //             if self.hat_left_range && distance < 16.into() {
        //                 sfx_player.catch();
        //                 self.hat_state = HatState::OnHead;
        //             }
        //         }
        //         HatState::OnHead => {
        //             // hat is on head, place hat on head
        //             self.hat_slow_counter = 0;
        //             self.hat_left_range = false;
        //             self.hat.position = self.warrior.position - hat_resting_position;
        //         }
        //         HatState::warriorTowards => {
        //             self.hat.sprite.set_sprite(
        //                 controller.sprite(hat_base_tile.animation_sprite(timer as usize / 2)),
        //             );
        //             let distance_vector =
        //                 self.hat.position - self.warrior.position + hat_resting_position;
        //             let distance = distance_vector.magnitude();
        //             if distance != 0.into() {
        //                 let v = self.warrior.velocity.magnitude() + 1;
        //                 self.warrior.velocity = distance_vector / distance * v;
        //             }
        //             self.warrior.velocity = self.warrior.update_position(level);
        //             if distance < 16.into() {
        //                 self.warrior.velocity /= 8;
        //                 self.hat_state = HatState::OnHead;
        //                 sfx_player.catch();
        //             }
        //         }
        //     }
        if !any_movement && self.is_on_ground {
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
