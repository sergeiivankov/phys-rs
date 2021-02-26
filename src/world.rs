#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use js_sys::Int32Array;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
#[cfg(target_arch = "wasm32")]
use instant::Instant;

use crate::body::{
  BodyId, BodiesIds, BodyClass, Body, Bodies,
  item::BodyItem, player::BodyPlayer
};
use crate::cells::Cells;
use crate::engine::{
  BLOCK_SIZE, EventClass, Event,
  PositionUpdate, Rects, UpdateResults,
  get_bounds_intersection, update_positions_typed, update_correct_players
};
use crate::grid::Grid;

#[cfg(not(target_arch = "wasm32"))]
#[derive(Default)]
pub struct World {
  width: i32,
  height: i32,
  next_body_id: u32,
  last_update: Option<Instant>,
  pub cells: Cells,
  pub grid: Grid,
  pub ids: BodiesIds,
  pub rects: Rects,
  pub items: Bodies<BodyItem>,
  pub players: Bodies<BodyPlayer>,
  ids_to_remove: BodiesIds
}

/**
 * Структура описана отдельно для wasm32,
 * потому что не получилось добавить wasm_bindgen(skip)
 * с условием архитектуры
 */
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
#[derive(Default)]
pub struct World {
  width: i32,
  height: i32,
  next_body_id: u32,
  last_update: Option<Instant>,
  #[wasm_bindgen(skip)]
  pub cells: Cells,
  #[wasm_bindgen(skip)]
  pub grid: Grid,
  #[wasm_bindgen(skip)]
  pub ids: BodiesIds,
  #[wasm_bindgen(skip)]
  pub rects: Rects,
  #[wasm_bindgen(skip)]
  pub items: Bodies<BodyItem>,
  #[wasm_bindgen(skip)]
  pub players: Bodies<BodyPlayer>,
  ids_to_remove: BodiesIds
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl World {
  #[cfg_attr(target_arch = "wasm32", wasm_bindgen(constructor))]
  pub fn new(width_blocks: i32, height_blocks: i32) -> Self {
    Self {
      width: width_blocks * BLOCK_SIZE,
      height: height_blocks * BLOCK_SIZE,
      cells: Cells::new(width_blocks, height_blocks),
      ..Default::default()
    }
  }

  pub(crate) fn next_body_id(&mut self) -> BodyId {
    self.next_body_id = self.next_body_id + 1;
    self.next_body_id
  }

  fn step_clear(&mut self) {
    for id in &self.ids_to_remove {
      let rect = match self.rects.remove(&id) {
        Some(rect) => rect,
        None => continue
      };

      match rect.class {
        BodyClass::Player => { self.players.remove(id); },
        BodyClass::Item => { self.items.remove(id); },
        _ => todo!()
      }

      self.grid.remove(&rect);
      self.rects.remove(id);
      self.ids.remove(&id);
    }

    self.ids_to_remove = BodiesIds::default();
  }

  fn step_update_positions(
    &mut self, delta: f32, events: &mut Vec<Event>
  ) {
    update_positions_typed(
      delta, self.width, self.height,
      &mut self.rects, &mut self.players,
      &mut self.ids_to_remove, events
    );
  }

  fn step_broadphase(&mut self) {
    for id in &self.ids {
      self.grid.update(*id, &mut self.rects);
    }
  }

  fn step_detect(&mut self, events: &mut Vec<Event>) {
    for pair in self.grid.pairs.values() {
      let rect1 = self.rects.get(&pair.id1).unwrap();
      let rect2 = self.rects.get(&pair.id2).unwrap();

      let intersection = get_bounds_intersection(
        &rect1.bounds, &rect2.bounds
      );

      if intersection.x <= 0 || intersection.y <= 0 {
        continue
      }

      match rect1.class {
        BodyClass::Sensor => {
          events.push(Event {
            class: EventClass::Sensor,
            body_id: rect2.id,
            trigger_id: rect1.id
          });
          continue
        },
        BodyClass::Item => match rect2.class {
          BodyClass::Player => {
            events.push(Event {
              class: EventClass::Item,
              body_id: rect2.id,
              trigger_id: rect1.id
            });
            continue
          }
          _ => ()
        },
        _ => ()
      }

      match rect2.class {
        BodyClass::Sensor => {
          events.push(Event {
            class: EventClass::Sensor,
            body_id: rect1.id,
            trigger_id: rect2.id
          });
          continue
        },
        BodyClass::Item => match rect1.class {
          BodyClass::Player => {
            events.push(Event {
              class: EventClass::Item,
              body_id: rect1.id,
              trigger_id: rect2.id
            });
            continue
          }
          _ => ()
        },
        _ => ()
      }
    }
  }

  fn step_correct(&mut self) {
    update_correct_players(
      &self.cells, &mut self.rects, &mut self.players
    );
  }

  fn step_finish(&mut self) -> Vec<PositionUpdate> {
    let mut positions_updates: Vec<PositionUpdate> = Vec::new();

    for (id, body) in self.players.iter_mut() {
      body.after_update();

      if body.x == body.prev_x && body.y == body.prev_y {
        continue
      }

      positions_updates.push(PositionUpdate {
        id: *id,
        x: body.x,
        y: body.y
      });

      body.prev_x = body.x;
      body.prev_y = body.y;

      body.update_rect(self.rects.get_mut(&id).unwrap());
    }

    positions_updates
  }

  fn _update(&mut self) -> UpdateResults {
    let delta = match self.last_update {
      Some(instant) => instant.elapsed().as_secs_f32(),
      None => 0.0
    };

    let mut events: Vec<Event> = Vec::new();

    if !self.ids_to_remove.is_empty() {
      self.step_clear();
    }

    self.step_update_positions(delta, &mut events);

    if !self.ids_to_remove.is_empty() {
      self.step_clear();
    }

    self.step_broadphase();

    self.step_detect(&mut events);

    self.step_correct();

    let positions_updates = self.step_finish();

    self.last_update = Some(Instant::now());

    UpdateResults {
      events: events,
      positions_updates: positions_updates
    }
  }

  #[cfg(not(target_arch = "wasm32"))]
  pub fn update(&mut self) -> UpdateResults {
    self._update()
  }

  #[cfg(target_arch = "wasm32")]
  pub fn update(&mut self) -> Int32Array {
    let update_results = self._update();

    let mut result = Vec::with_capacity(
      (update_results.events.len() * 3 +
       update_results.positions_updates.len() * 3 +
       3) as usize
    );

    for event in &update_results.events {
      result.push(event.class as i32);
      result.push(event.body_id as i32);
      result.push(event.trigger_id as i32);
    }

    result.push(0);
    result.push(0);
    result.push(0);

    for position_update in &update_results.positions_updates {
      result.push(position_update.id as i32);
      result.push(position_update.x);
      result.push(position_update.y);
    }

    Int32Array::from(&result[..])
  }

  pub fn remove(&mut self, id: BodyId) {
    self.ids_to_remove.insert(id);
  }

  fn _get_possible_build_blocks(&self, player_id: BodyId) -> Vec<i32> {
    // 14 - максимальное число возможных ячеек для постройки блока
    // 14 * 2 = 28 координат
    // На практике реальное количество будет меньше,
    // в будущем провести статанализ и найти оптимальное значение запаса
    let mut result = Vec::with_capacity(28);

    let rect = match self.rects.get(&player_id) {
      Some(rect) => rect,
      None => return result
    };

    let min_x = (rect.bounds.min_x >> 7) - 1;
    let max_x = (rect.bounds.max_x >> 7) + 1;
    let min_y = (rect.bounds.min_y >> 7) - 1;
    let max_y = (rect.bounds.max_y >> 7) + 1;

    for x_cell in min_x..=max_x {
      if self.cells.can_build(x_cell, min_y) {
        result.push(x_cell);
        result.push(min_y);
      }

      if self.cells.can_build(x_cell, max_y) {
        result.push(x_cell);
        result.push(max_y);
      }
    }

    for y_cell in (min_y + 1)..=(max_y - 1) {
      if self.cells.can_build(min_x, y_cell) {
        result.push(min_x);
        result.push(y_cell);
      }

      if self.cells.can_build(max_x, y_cell) {
        result.push(max_x);
        result.push(y_cell);
      }
    }

    result
  }

  #[cfg(not(target_arch = "wasm32"))]
  pub fn get_possible_build_blocks(&self, player_id: BodyId) -> Vec<i32> {
    self._get_possible_build_blocks(player_id)
  }

  #[cfg(target_arch = "wasm32")]
  pub fn get_possible_build_blocks(&self, player_id: BodyId) -> Int32Array {
    Int32Array::from(&self._get_possible_build_blocks(player_id)[..])
  }
}