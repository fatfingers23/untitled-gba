use crate::level::Level;
use crate::player::WARRIOR_IDLE;
use crate::types::FixedNumberType;
use agb::display::object::{OamManaged, Object};
use agb::display::{Priority, HEIGHT, WIDTH};
use agb::fixnum::Vector2D;
use agb::println;

pub struct Entity<'a> {
    pub sprite: Object<'a>,
    pub size: Vector2D<i32>,
    pub position: Vector2D<FixedNumberType>,
    pub velocity: Vector2D<FixedNumberType>,
    pub collision_mask: Vector2D<u16>,
    ///Only applies to the sprite, not the collision mask or position
    pub sprite_offset: Vector2D<i32>,
}

impl<'a> Entity<'a> {
    pub fn new(
        object: &'a OamManaged,
        collision_mask: Vector2D<u16>,
        offset: Option<Vector2D<i32>>,
    ) -> Self {
        let mut dummy_object = object.object_sprite(WARRIOR_IDLE.sprites().first().unwrap());
        dummy_object.set_priority(Priority::P1);
        Entity {
            sprite: dummy_object,
            collision_mask,
            size: (16, 16).into(),
            position: (0, 0).into(),
            velocity: (0, 0).into(),
            sprite_offset: offset.unwrap_or(Vector2D::new(0, 0)),
        }
    }

    fn something_at_point<T: Fn(i32, i32) -> bool>(
        &self,
        position: Vector2D<FixedNumberType>,
        something_fn: T,
    ) -> bool {
        let left = (position.x - self.collision_mask.x as i32 / 2).floor() / 8;
        let right = (position.x + self.collision_mask.x as i32 / 2 - 1).floor() / 8;
        let top = (position.y - self.collision_mask.y as i32 / 2).floor() / 8;
        let bottom = (position.y + self.collision_mask.y as i32 / 2 - 1).floor() / 8;

        for x in left..=right {
            for y in top..=bottom {
                if something_fn(x, y) {
                    return true;
                }
            }
        }
        false
    }

    pub fn collision_at_point(&self, level: &Level, position: Vector2D<FixedNumberType>) -> bool {
        self.something_at_point(position, |x, y| level.collides(x, y))
    }

    pub(crate) fn killision_at_point(
        &self,
        level: &Level,
        position: Vector2D<FixedNumberType>,
    ) -> bool {
        self.something_at_point(position, |x, y| level.kills(x, y))
    }

    pub(crate) fn completion_at_point(
        &self,
        level: &Level,
        position: Vector2D<FixedNumberType>,
    ) -> bool {
        self.something_at_point(position, |x, y| level.wins(x, y))
    }

    // fn enemy_collision_at_point(
    //     &self,
    //     enemies: &[enemies::Enemy],
    //     position: Vector2D<FixedNumberType>,
    // ) -> bool {
    //     for enemy in enemies {
    //         if enemy.collides_with_hat(position) {
    //             return true;
    //         }
    //     }
    //     false
    // }

    // returns the distance actually moved
    pub fn update_position(&mut self, level: &Level) -> Vector2D<FixedNumberType> {
        let old_position = self.position;
        let x_velocity = (self.velocity.x, 0.into()).into();
        if !self.collision_at_point(level, self.position + x_velocity) {
            self.position += x_velocity;
        } else {
            self.position += self.binary_search_collision(level, (1, 0).into(), self.velocity.x);
        }

        let y_velocity = (0.into(), self.velocity.y).into();
        if !self.collision_at_point(level, self.position + y_velocity) {
            self.position += y_velocity;
        } else {
            self.position += self.binary_search_collision(level, (0, 1).into(), self.velocity.y);
        }
        self.position - old_position
    }

    // fn update_position_with_enemy(
    //     &mut self,
    //     level: &Level,
    //     enemies: &[enemies::Enemy],
    // ) -> (Vector2D<FixedNumberType>, bool) {
    //     let mut was_enemy_collision = false;
    //     let old_position = self.position;
    //     let x_velocity = (self.velocity.x, 0.into()).into();
    //
    //     if !(self.collision_at_point(level, self.position + x_velocity)
    //         || self.enemy_collision_at_point(enemies, self.position + x_velocity))
    //     {
    //         self.position += x_velocity;
    //     } else if self.enemy_collision_at_point(enemies, self.position + x_velocity) {
    //         self.position -= x_velocity;
    //         was_enemy_collision = true;
    //     }
    //
    //     let y_velocity = (0.into(), self.velocity.y).into();
    //     if !(self.collision_at_point(level, self.position + y_velocity)
    //         || self.enemy_collision_at_point(enemies, self.position + y_velocity))
    //     {
    //         self.position += y_velocity;
    //     } else if self.enemy_collision_at_point(enemies, self.position + y_velocity) {
    //         self.position -= y_velocity;
    //         was_enemy_collision = true;
    //     }
    //
    //     (self.position - old_position, was_enemy_collision)
    // }

    fn binary_search_collision(
        &self,
        level: &Level,
        unit_vector: Vector2D<FixedNumberType>,
        initial: FixedNumberType,
    ) -> Vector2D<FixedNumberType> {
        let mut low: FixedNumberType = 0.into();
        let mut high = initial;

        let one: FixedNumberType = 1.into();
        while (high - low).abs() > one / 8 {
            let mid = (low + high) / 2;
            let new_vel: Vector2D<FixedNumberType> = unit_vector * mid;

            if self.collision_at_point(level, self.position + new_vel) {
                high = mid;
            } else {
                low = mid;
            }
        }

        unit_vector * low
    }

    pub fn commit_position(&mut self, additional_offset: Vector2D<FixedNumberType>) {
        let mut position = (self.position - additional_offset).floor();
        let position =
            position - Vector2D::new(self.sprite_offset.x.into(), self.sprite_offset.y.into());
        self.sprite.set_position(position - self.size);
        if position.x < -self.size.x
            || position.x > WIDTH + self.size.x
            || position.y < -self.size.y
            || position.y > HEIGHT + self.size.y
        {
            self.sprite.hide();
        } else {
            self.sprite.show();
        }
    }
}
