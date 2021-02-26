use std::collections::{ HashMap, HashSet };
use crate::body::{ BodyId, BodyClass };
use crate::engine::{ Bounds, Rect, Rects, RegionId, RegionsIds };

type PairId = u64;

#[derive(Default, Debug)]
pub struct Pair {
  pub id1: BodyId,
  pub id2: BodyId,
  // Количество связей пары, необходимо, так как
  // оба тела могут иметь возможность пересекаться
  // в нескольких регионах
  pub count: u8
}

/**
 * Возвращает идентификатор пары (64 bit) для двух идентификаторов тел (32 bit)
 */
fn get_pair_id(id1: BodyId, id2: BodyId) -> PairId {
  let id1_num = id1 as u64;
  let id2_num = id2 as u64;

  // Используется сдвиг в 26 бит, так как в JS максимальная
  // безопасная точность числа равна 53 битам, то есть,
  // на выходе необходимо получить 64 битное число,
  // но использовать не более 53 бит.
  // Таким образом, максимальное значение идентификатора тела
  // равно 2**26 - 1 = 67108863
  if id1_num < id2_num {
    (id1_num << 26) + id2_num
  } else {
    (id2_num << 26) + id1_num
  }
}

/**
 * Битовые маски классов тел
 */
const BODIES_CATEGORIES: [u8; 6] = [
  0b00000001,
  0b00000010,
  0b00000100,
  0b00001000,
  0b00010000,
  0b00100000
];

/**
 * Битовые фильтры возможности столкновений классов тел
 */
const BODIES_FILTERS: [u8; 6] = [
  0b00111100,
  0b00000100,
  0b00111011,
  0b00000101,
  0b00000101,
  0b00000101
];

/**
 * Определяет возможность столкновения тел, в зависимости от их класса
 *
 * Таблица возможности столкновений классов тел:
 * +--------+--------+--------+--------+--------+--------+--------+
 * |        | Fixed  | Sensor | Player | Ray    | Item   | Bullet |
 * +--------+--------+--------+--------+--------+--------+--------+
 * | Fixed  |        |        |   XX   |   XX   |   XX   |   XX   |
 * | Sensor |        |        |   XX   |        |        |        |
 * | Player |   XX   |   XX   |        |   XX   |   XX   |   XX   |
 * | Ray    |   XX   |        |   XX   |        |        |        |
 * | Item   |   XX   |        |   XX   |        |        |        |
 * | Bullet |   XX   |        |   XX   |        |        |        |
 * +--------+--------+--------+--------+--------+--------+--------+
 */
fn can_collide(class1: &BodyClass, class2: &BodyClass) -> bool {
  if class1 == class2 {
    return false
  }

  let category = BODIES_CATEGORIES[*class1 as usize];
  let filter = BODIES_FILTERS[*class2 as usize];

  if category & filter == 0 {
    return false
  }

  true
}

/**
 * Возвращает массив идентификаторов регионов по ограничительному прямоугольнику
 *
 * Карта разбивается на квадраты (регионы) со стороной 1024 точки
 * (используется смещение >> на 10 бит).
 * Для идентификатора региона (включающего координаты региона по осям)
 * используется 32 бита, таким образом, максимальная ширина и высота
 * в регионах равна 65534 (16 бит) (почему не 65535 см. в комментари в коде),
 * в точках равна 67106816 (65534 * 1024).
 */
fn get_regions_by_bounds(bounds: &Bounds) -> RegionsIds {
  let mut regions = RegionsIds::default();

  // Тело может находится в от 1 до 4 регионов, таким образом,
  // необходимо в результирующем массиве (фиксированной длинны = 4)
  // определять пустые элементы. Для пустых элементов зарезирвирован ноль,
  // поэтому идентификатор региона не может равняться нулю, поэтому к
  // "x" координатам добавляется 1.
  let min_x = (bounds.min_x >> 10) + 1;
  let max_x = (bounds.max_x >> 10) + 1;
  let min_y = bounds.min_y >> 10;
  let max_y = bounds.max_y >> 10;

  let mut index = 0;
  for x in min_x..=max_x {
    for y in min_y..=max_y {
      regions[index] = (y << 16) + x;
      index += 1;
    }
  }

  regions
}

/**
 * Сетка
 *
 * Необходима для разделения физического мира на регионы
 * и определения пар тел с возможностью столкновения только
 * назодящихся в одном регионе.
 *
 * Один регион имеет размер 1024 на 1024 пунктов.
 * Максимум регионов по ширине и высоте 256 (см. описание
 * функции get_regions_by_bounds)
 */
#[derive(Default)]
pub struct Grid {
  // Пары идентификаторов тел с возможностью столкновения
  pub pairs: HashMap<PairId, Pair>,
  // Списки идентификаторов объектов, разбитых по регионам
  hash: HashMap<RegionId, HashSet<BodyId>>
}

const EMPTY_REGION: i32 = 0;

impl Grid {
  fn add_to_pairs(
    &mut self, regions: &RegionsIds, id: BodyId, rects: &mut Rects
  ) {
    let rect = rects.get(&id).unwrap();

    for region in regions {
      if region == &EMPTY_REGION {
        break;
      }

      for other_id in self.hash.get(region).unwrap() {
        if rect.id == *other_id {
          continue
        }

        let other_rect = rects.get(other_id).unwrap();

        if !can_collide(&rect.class, &other_rect.class) {
          continue
        }

        let pair_id = get_pair_id(rect.id, other_rect.id);

        if self.pairs.contains_key(&pair_id) {
          let pair = self.pairs.get_mut(&pair_id).unwrap();
          pair.count = pair.count + 1;
        } else {
          self.pairs.insert(pair_id, Pair {
            id1: rect.id,
            id2: other_rect.id,
            count: 1
          });
        }
      }
    }
  }

  fn remove_from_pairs(&mut self, regions: &RegionsIds, id: BodyId) {
    for region in regions {
      if region == &EMPTY_REGION {
        break;
      }

      for other_id in self.hash.get(region).unwrap() {
        let pair_id = get_pair_id(id, *other_id);

        match self.pairs.get_mut(&pair_id) {
          None => continue,
          Some(pair) => {
            if pair.count == 1 {
              self.pairs.remove(&pair_id);
            } else {
              pair.count = pair.count - 1;
            }
          }
        }
      }
    }
  }

  /**
   * Добавление тела в сетку
   */
  pub fn add(&mut self, id: BodyId, rects: &mut Rects) {
    let mut rect = rects.get_mut(&id).unwrap();

    let regions = get_regions_by_bounds(&rect.bounds);
    rect.regions = regions;

    for region in &regions {
      if region == &EMPTY_REGION {
        break;
      }

      if !self.hash.contains_key(region) {
        self.hash.insert(*region, HashSet::new());
      }
      self.hash.get_mut(region).unwrap().insert(rect.id);
    }

    self.add_to_pairs(&regions, id, rects);
  }

  /**
   * Обновление тела в сетке
   */
  pub fn update(&mut self, id: BodyId, rects: &mut Rects) {
    let mut rect = rects.get_mut(&id).unwrap();

    if !rect.is_updated {
      return
    }
    rect.is_updated = false;

    let new_regions = get_regions_by_bounds(&rect.bounds);
    let old_regions = rect.regions;

    if new_regions == old_regions {
      return
    }

    rect.regions = new_regions;

    let mut regions_to_remove = RegionsIds::default();

    let mut regions_to_remove_count: usize = 0;
    for region in &old_regions {
      if region == &EMPTY_REGION {
        break;
      }

      if !new_regions.contains(region) {
        regions_to_remove[regions_to_remove_count] = *region;
        regions_to_remove_count += 1;

        self.hash.get_mut(region).unwrap().remove(&rect.id);
      }
    }

    if regions_to_remove_count > 0 {
      self.remove_from_pairs(&regions_to_remove, id);
    }

    let mut regions_to_add = RegionsIds::default();

    let mut regions_to_add_count: usize = 0;
    for region in &new_regions {
      if region == &EMPTY_REGION {
        break;
      }

      if !old_regions.contains(region) {
        regions_to_add[regions_to_add_count] = *region;
        regions_to_add_count += 1;

        if !self.hash.contains_key(region) {
          self.hash.insert(*region, HashSet::new());
        }
        self.hash.get_mut(region).unwrap().insert(rect.id);
      }
    }

    if regions_to_add_count > 0 {
      self.add_to_pairs(&regions_to_add, id, rects);
    }
  }

  /**
   * Удаление тела из сетки
   */
  pub fn remove(&mut self, rect: &Rect) {
    for region in &rect.regions {
      if region == &EMPTY_REGION {
        break;
      }

      self.hash.get_mut(region).unwrap().remove(&rect.id);
    }

    self.remove_from_pairs(&rect.regions, rect.id);
  }
}