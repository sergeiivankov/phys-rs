#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use std::cmp::Ordering;
use crate::body::{ BodyId, BodyClass, Body };
use crate::engine::{ Direction, Rect, Vector };
use crate::world::World;

pub const BODY_PLAYER_WIDTH: i32 = 64;
pub const BODY_PLAYER_HALF_WIDTH: i32 = BODY_PLAYER_WIDTH / 2;
pub const BODY_PLAYER_HEIGHT: i32 = 208;
pub const BODY_PLAYER_GRAVITY: f32 = 0.001;
// Максимальная высота прыжка
//pub const BODY_PLAYER_JUMP_DISTANCE: i32 = 320;
pub const BODY_PLAYER_JUMP_DISTANCE: i32 = 160;
// Коэффициент расчета расстояния в прыжке
// = sqrt(BODY_PLAYER_JUMP_DISTANCE / BODY_PLAYER_GRAVITY)
pub const BODY_PLAYER_JUMP_COEF: f32 = 400.0;
pub const BODY_PLAYER_MOVE_SPEED: f32 = 750.0;

#[derive(Default, Debug)]
pub struct BodyPlayer {
  pub x: i32,
  pub y: i32,
  pub prev_x: i32,
  pub prev_y: i32,
  pub force_x: f32,
  last_ground_y: i32,
  pub is_jump: bool,
  jump_timer: f32,
  jump_x_decreased: bool,
  jump_x_setted: bool,
  pub is_fall: bool,
  fall_timer: f32,
  move_dir_y: i8,
  is_on_ground: bool,
  move_state: Direction,
  current_tick_corrected: bool
}

impl BodyPlayer {
  pub fn new(x: i32, y: i32) -> Self {
    Self {
      x: x,
      y: y,
      prev_x: x,
      prev_y: y,
      last_ground_y: y,
      ..Default::default()
    }
  }

  pub fn update_correction(&mut self, correction: &Vector) {
    self.current_tick_corrected = true;

    if correction.x != 0 && !self.is_on_ground && !self.jump_x_decreased {
      self.force_x /= 2.0;
      self.jump_x_decreased = true;
    }

    match correction.y.cmp(&0) {
      Ordering::Less => {
        self.is_on_ground = true;
        self.is_jump = false;
        self.jump_timer = 0.0;
        self.jump_x_decreased = false;
        self.jump_x_setted = false;
        self.is_fall = false;
        self.fall_timer = 0.0;

        match self.move_state {
          Direction::None => self.force_x = 0.0,
          _ => {
            let move_state = self.move_state;
            self.move_state = Direction::None;
            self.run(move_state);
          }
        };

        /*if self.force_x != 0 && self.is_run {
          self.run(if self.force_x > 0 { 1 } else { -1 });
        } else {
          self.force_x = 0;
        }*/
      },
      Ordering::Greater => {
        self.is_jump = false;
        self.jump_timer = 0.0;
      },
      Ordering::Equal => {
        self.is_on_ground = false;
      }
    }
  }

  pub fn after_update(&mut self) {
    if !self.current_tick_corrected {
      self.is_on_ground = false;
    }

    if !self.is_on_ground
    && !self.is_jump
    && !self.is_fall {
      self.is_fall = true;
      self.fall_timer = 0.0;
      self.last_ground_y = self.y;

      self.jump_x_setted = match self.move_state {
        Direction::None => false,
        _ => true
      };

      let direction_num = match self.move_state {
        Direction::None => return,
        Direction::Left => -1.0,
        Direction::Right => 1.0,
      };
      self.force_x = BODY_PLAYER_MOVE_SPEED * direction_num;
    }
  }

  pub fn run(&mut self, direction: Direction) {
    if self.move_state == direction {
      return
    }

    self.move_state = direction;

    if !self.is_on_ground {
      if !self.jump_x_setted {
        self.jump_x_setted = true;
        self.jump_x_decreased = true;

        let direction_num = match direction {
          Direction::None => 0.0,
          Direction::Left => -1.0,
          Direction::Right => 1.0,
        };
        self.force_x = BODY_PLAYER_MOVE_SPEED * direction_num / 2.0;
      }

      return
    }

    let direction_num = match direction {
      Direction::None => 0.0,
      Direction::Left => -1.0,
      Direction::Right => 1.0,
    };
    self.force_x = BODY_PLAYER_MOVE_SPEED * direction_num;
  }

  pub fn jump(&mut self) {
    if !self.is_on_ground {
      return
    }

    self.is_jump = true;
    self.jump_timer = 0.0;

    self.last_ground_y = self.y;

    self.is_on_ground = false;

    self.jump_x_setted = match self.move_state {
      Direction::None => false,
      _ => true
    }
  }
}

impl Body for BodyPlayer {
  fn update(&mut self, delta: f32, rect: &mut Rect) {
    //delta = delta / 4.0;
    self.current_tick_corrected = false;

    if self.is_on_ground {
      rect.is_updated = true;
      self.y += 1;
      //self.is_on_ground = false;
    }

    if self.force_x != 0.0 {
      rect.is_updated = true;

      // IMPORTANT: может быть проблема из-за округления
      // при конвертации в i32 тип с отбрасыванием дробной части
      // (в виде уменьшения реальной скорости)
      // При использовании округления .round(), будет аналогичный
      // эффект с нестабильным уменьшением/увеличением скорости
      self.x += (self.force_x * delta) as i32;

      //if self.is_on_ground {
      //  self.y += 1;
        //self.is_on_ground = false;
      //}
    }

    self.move_dir_y = 0;

    if self.is_jump {
      self.jump_timer += delta * 1000.0;

      // IMPORTANT: может быть проблема из-за округления
      // при конвертации в i32 тип с отбрасыванием дробной части
      // (в виде уменьшения реальной скорости)
      // При использовании округления .round(), будет аналогичный
      // эффект с нестабильным уменьшением/увеличением скорости
      self.y = self.last_ground_y
        + (BODY_PLAYER_GRAVITY * (self.jump_timer - BODY_PLAYER_JUMP_COEF).powf(2.0)) as i32
        - BODY_PLAYER_JUMP_DISTANCE;

      self.move_dir_y = if self.jump_timer - BODY_PLAYER_JUMP_COEF > 0.0 { 1 } else { -1 };

      rect.is_updated = true;
    }

    if self.is_fall {
      self.fall_timer += delta * 1000.0;

      // IMPORTANT: может быть проблема из-за округления
      // при конвертации в i32 тип с отбрасыванием дробной части
      // (в виде уменьшения реальной скорости)
      // При использовании округления .round(), будет аналогичный
      // эффект с нестабильным уменьшением/увеличением скорости
      self.y = self.last_ground_y
        + (BODY_PLAYER_GRAVITY * self.fall_timer.powf(2.0)) as i32;

      self.move_dir_y = 1;

      rect.is_updated = true;
    }

    if rect.is_updated {
      self.update_rect(rect);
    }
  }

  fn update_rect(&mut self, rect: &mut Rect) {
    rect.bounds.min_x = self.x - BODY_PLAYER_HALF_WIDTH;
    rect.bounds.max_x = self.x + BODY_PLAYER_HALF_WIDTH;
    rect.bounds.min_y = self.y - BODY_PLAYER_HEIGHT;
    rect.bounds.max_y = self.y;
  }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl World {
  pub fn player_create(&mut self, x: i32, y: i32) -> BodyId {
    let id = self.next_body_id();

    self.rects.insert(id, Rect::new(
      id, BodyClass::Player, x, y, BODY_PLAYER_HALF_WIDTH, BODY_PLAYER_HEIGHT
    ));
    self.grid.add(id, &mut self.rects);

    self.players.insert(id, BodyPlayer::new(x, y));

    self.ids.insert(id);

    id
  }

  pub fn player_run(&mut self, id: BodyId, direction: Direction) {
    match self.players.get_mut(&id) {
      Some(player) => player.run(direction),
      None => ()
    };
  }

  pub fn player_jump(&mut self, id: BodyId) {
    match self.players.get_mut(&id) {
      Some(player) => player.jump(),
      None => ()
    };
  }
}