/**
 * Тип тела Block не имеет отельного объекта с дополнительными
 * свойствами, наличие тела Block определяется по занятости
 * клетки в поле blocks в структуре ячеек Cells
 */

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use crate::world::World;
use crate::body::BodyId;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl World {
  /**
   * Создает Block тело
   */
  pub fn block_create(&mut self, x: i32, y: i32) -> BodyId {
    if x > self.cells.width || y > self.cells.height {
      return 0
    }

    self.cells.set_block(x, y, true);

    let id = y * self.cells.width + x + 1;

    id as u32
  }

  /**
   * Удаляет Block тело
   */
  pub fn block_remove(&mut self, id: BodyId) {
    let id = id as i32 - 1;

    let x = id % self.cells.width;
    let y = id / self.cells.width;

    self.cells.set_block(x, y, false);
  }
}