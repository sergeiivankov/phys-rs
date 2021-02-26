pub mod block;
pub mod item;
pub mod player;

use std::collections::{ HashMap, HashSet };
use crate::engine::Rect;

pub type BodyId = u32;

pub type BodiesIds = HashSet<BodyId>;

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum BodyClass {
  Fixed = 0,
  Sensor = 1,
  Player = 2,
  Ray = 3,
  Item = 4,
  Bullet = 5
}

pub trait Body {
  fn update(&mut self, delta: f32, rect: &mut Rect);
  fn update_rect(&mut self, rect: &mut Rect);
}

pub type Bodies<Body> = HashMap<BodyId, Body>;