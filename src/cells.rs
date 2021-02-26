use std::mem::size_of;

/**
 * Структура, хранящая информацию о временно занятых
 * телами ячеек и занятых Block телами ячеек.
 *
 * Необходима для определения возможности строительства в ячейке и
 * проверки столкновений с Block телами.
 *
 * Ячейка - квардрат размера одного Block тела (128 на 128 пунктов).
 *
 * Для оптимизации использования памяти данных хранятся в определенных
 * битах массива usize чисел
 */
#[derive(Default)]
pub struct Cells {
  pub width: i32,
  pub height: i32,
  busy: Vec<usize>,
  blocks: Vec<usize>
}

/**
 * Определение количества бит в целевой системе
 */
const TARGET_BITS: usize = size_of::<usize>() * 8;

impl Cells {
  /**
   * Инициализация блоков, заполнение векторов со статусами занятости
   * и занятости Block телом по ширине и высоте карты
   */
  pub fn new(width: i32, height: i32) -> Self {
    let size = ((width * height) as f32 / TARGET_BITS as f32).ceil() as usize;

    Self {
      width,
      height,
      busy: vec![0; size],
      blocks: vec![0; size]
    }
  }

  /**
   * Установка ячейке статуса занятой
   */
  /*pub fn set_busy(&mut self, x: i32, y: i32, state: bool) {
    let index = (y * self.width + x) as usize;
    let pos = index / TARGET_BITS;
    let bit = index % TARGET_BITS;

    if state {
      self.busy[pos] |= 1 << bit;
    } else {
      self.busy[pos] &= 1 << bit ^ usize::MAX;
    };
  }*/

  /**
   * Установка ячейке статуса занятой Block телом
   */
  pub fn set_block(&mut self, x: i32, y: i32, state: bool) {
    let index = (y * self.width + x) as usize;
    let pos = index / TARGET_BITS;
    let bit = index % TARGET_BITS;

    if state {
      self.busy[pos] |= 1 << bit;
      self.blocks[pos] |= 1 << bit;
    } else {
      self.busy[pos] &= 1 << bit ^ usize::MAX;
      self.blocks[pos] &= 1 << bit ^ usize::MAX;
    };
  }

  /**
   * Проверка статуса ячейки на занятость
   */
  pub fn is_busy(&self, x: i32, y: i32) -> bool {
    if x < 0 || x >= self.width || y < 0 || y >= self.height {
      return true
    }

    let index = (y * self.width + x) as usize;
    let pos = index / TARGET_BITS;
    let bit = index % TARGET_BITS;

    self.busy[pos] & (1 << bit) != 0
  }

  /**
   * Проверка статуса ячейки, занята ли Block телом
   */
  pub fn is_block(&self, x: i32, y: i32) -> bool {
    if x < 0 || x >= self.width || y < 0 || y >= self.height {
      return true
    }

    let index = (y * self.width + x) as usize;
    let pos = index / TARGET_BITS;
    let bit = index % TARGET_BITS;

    self.blocks[pos] & (1 << bit) != 0
  }

  pub fn can_build(&self, x: i32, y: i32) -> bool {
    !self.is_busy(x, y) &&
    (self.is_block(x - 1, y) || self.is_block(x + 1, y) ||
     self.is_block(x, y - 1) || self.is_block(x, y + 1))
  }
}