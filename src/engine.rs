#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use std::cmp::{ min, max };
use std::collections::HashMap;
use crate::body::{ BodyId, BodiesIds, BodyClass, Body, Bodies };
use crate::body::player::{ BODY_PLAYER_HALF_WIDTH, BODY_PLAYER_HEIGHT, BodyPlayer };
use crate::cells::Cells;

/**
 * Размер одного Fixed блока
 * Используется как единица измерения
 */
pub const BLOCK_SIZE: i32 = 128;
pub const BLOCK_HALF_SIZE: i32 = BLOCK_SIZE / 2;

/**
 * Ограничительный прямоугольник тела
 * Содержит максимальные и минимальные координаты
 * сторон прямоугольника
 */
#[derive(Debug)]
pub struct Bounds {
  pub min_x: i32,
  pub max_x: i32,
  pub min_y: i32,
  pub max_y: i32
}

/**
 * Напрявления движения тела
 */
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Direction {
  None = 0,
  Left = 1,
  Right = 2
}

impl Default for Direction {
  fn default() -> Self {
    Self::None
  }
}

/**
 * Тип события
 */
#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum EventClass {
  // Тело вышло за границы мира
  OutOfWorld = 0,
  // Пересечение тела игрока с сенсором
  Sensor = 1,
  // Пересечение тела игрока с телом предмета
  Item = 2
}

/**
 * Событие физического мира
 */
#[derive(Debug)]
pub struct Event {
  // Тип события
  pub class: EventClass,
  // Идентификатор тела, для которого сработало событие
  pub body_id: BodyId,
  // Идентификатор тела - причины события
  pub trigger_id: BodyId
}

/**
 * Обновление позиции тела
 */
#[derive(Debug)]
pub struct PositionUpdate {
  pub id: BodyId,
  pub x: i32,
  pub y: i32
}

/**
 * Прямоугольник тела для сетки
 */
#[derive(Debug)]
pub struct Rect {
  pub id: BodyId,
  pub class: BodyClass,
  pub bounds: Bounds,
  pub regions: RegionsIds,
  pub is_updated: bool
}

impl Rect {
  pub fn new(
    id: u32, class: BodyClass, x: i32, y: i32, half_width: i32, height: i32
  ) -> Self {
    Self {
      id: id,
      class: class,
      bounds: Bounds {
        min_x: x - half_width,
        max_x: x + half_width,
        min_y: y - height,
        max_y: y
      },
      regions: RegionsIds::default(),
      is_updated: false
    }
  }
}

pub type Rects = HashMap<BodyId, Rect>;

/**
 * Идентификатор региона и группа идентификаторов
 */
pub type RegionId = i32;
pub type RegionsIds = [RegionId; 4];

/**
 * Результаты обновления физического мира
 */
#[derive(Debug)]
pub struct UpdateResults {
  // Список событий
  pub events: Vec<Event>,
  // Список изменений позиций
  pub positions_updates: Vec<PositionUpdate>
}

/**
 * Самый, насколько это возможно, просто вектор
 */
pub struct Vector {
  pub x: i32,
  pub y: i32
}

/**
 * Возвращает вектор пересечения двух ограничительный прямоугольников
 */
pub fn get_bounds_intersection(bounds1: &Bounds, bounds2: &Bounds) -> Vector {
  let max_min_x = max(bounds1.min_x, bounds2.min_x);
  let min_max_x = min(bounds1.max_x, bounds2.max_x);

  let max_min_y = max(bounds1.min_y, bounds2.min_y);
  let min_max_y = min(bounds1.max_y, bounds2.max_y);

  Vector {
    x: min_max_x - max_min_x,
    y: min_max_y - max_min_y,
  }
}

pub fn update_positions_typed<T: Body>(
  delta: f32, world_width: i32, world_height: i32,
  rects: &mut Rects, bodies: &mut Bodies<T>,
  ids_to_remove: &mut BodiesIds, events: &mut Vec<Event>
) {
  for (id, body) in bodies.iter_mut() {
    let mut rect = rects.get_mut(id).unwrap();

    body.update(delta, &mut rect);

    if rect.bounds.min_x < 0
    || rect.bounds.max_x > world_width
    || rect.bounds.min_y < 0
    || rect.bounds.max_y > world_height {
      ids_to_remove.insert(*id);
      events.push(Event {
        class: EventClass::OutOfWorld,
        body_id: *id,
        trigger_id: 0
      });
    }
  }
}

pub fn update_correct_players(
  cells: &Cells, rects: &mut Rects,
  players: &mut Bodies<BodyPlayer>
) {
  for (id, player_body) in players.iter_mut() {
    let rect = rects.get_mut(id).unwrap();

    let min_x = rect.bounds.min_x >> 7;
    let max_x = rect.bounds.max_x >> 7;
    let min_y = rect.bounds.min_y >> 7;
    let max_y = rect.bounds.max_y >> 7;

    let mut correction = Vector { x: 0, y: 0 };

    for x_cell in min_x..=max_x {
      for y_cell in min_y..=max_y {
        if !cells.is_block(x_cell, y_cell) {
          continue
        }

        let x = x_cell * BLOCK_SIZE;
        let y = y_cell * BLOCK_SIZE;
        let block_bounds = Bounds {
          min_x: x,
          max_x: x + BLOCK_SIZE,
          min_y: y,
          max_y: y + BLOCK_SIZE
        };

        let intersection = get_bounds_intersection(
          &rect.bounds, &block_bounds
        );
        if intersection.x <= 0 || intersection.y <= 0 {
          continue
        }

        let mut correction_x = intersection.x;
        let mut correction_y = intersection.y;

        if rect.bounds.max_y < block_bounds.max_y {
          correction_y = -correction_y;
        }
        if player_body.x < x + BLOCK_HALF_SIZE {
          correction_x = -correction_x;
        }

        let prev_bounds = Bounds {
          min_x: player_body.prev_x - BODY_PLAYER_HALF_WIDTH,
          max_x: player_body.prev_x + BODY_PLAYER_HALF_WIDTH,
          min_y: player_body.prev_y - BODY_PLAYER_HEIGHT,
          max_y: player_body.prev_y
        };

        let prev_intersection = get_bounds_intersection(
          &prev_bounds, &block_bounds
        );

        if prev_intersection.x > 0 {
          correction_x = 0;
        }
        else if prev_intersection.y > 0 {
          correction_y = 0;
        }
        else {
          if player_body.force_x != 0.0 {
            correction_x = 0;
          }

          if player_body.is_fall || player_body.is_jump {
            correction_y = 0;
          }
        }

        if correction_x.abs() > correction.x.abs() {
          correction.x = correction_x;
        }
        if correction_y.abs() > correction.y.abs() {
          correction.y = correction_y;
        }
      }
    }

    if correction.x == 0 && correction.y == 0 {
      continue
    }

    player_body.update_correction(&correction);

    let new_x = player_body.x + correction.x;
    let new_y = player_body.y + correction.y;

    player_body.x = new_x;
    player_body.y = new_y;

    rect.bounds.min_x = new_x - BODY_PLAYER_HALF_WIDTH;
    rect.bounds.max_x = new_x + BODY_PLAYER_HALF_WIDTH;
    rect.bounds.min_y = new_y - BODY_PLAYER_HEIGHT;
    rect.bounds.max_y = new_y;
  }
}