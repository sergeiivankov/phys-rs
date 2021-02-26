#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use crate::body::{ BodyId, BodyClass, Body };
use crate::engine::Rect;
use crate::world::World;

const BODY_ITEM_WIDTH: i32 = 64;
const BODY_ITEM_HALF_WIDTH: i32 = BODY_ITEM_WIDTH / 2;
const BODY_ITEM_HEIGHT: i32 = 64;

pub struct BodyItem {
  pub x: i32,
  pub y: i32,
  pub move_dir_y: i8,
  pub is_on_ground: bool
}

impl BodyItem {
  pub fn new(x: i32, y: i32) -> Self {
    Self {
      x: x,
      y: y,
      move_dir_y: 0,
      is_on_ground: false
    }
  }
}

impl Body for BodyItem {
  fn update(&mut self, _delta: f32, _rect: &mut Rect) {

  }

  fn update_rect(&mut self, _rect: &mut Rect) {

  }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl World {
  pub fn item_create(&mut self, x: i32, y: i32) -> BodyId {
    let id = self.next_body_id();

    self.rects.insert(id, Rect::new(
      id, BodyClass::Item, x, y, BODY_ITEM_HALF_WIDTH, BODY_ITEM_HEIGHT
    ));
    self.grid.add(id, &mut self.rects);

    self.items.insert(id, BodyItem::new(x, y));

    self.ids.insert(id);

    id
  }
}