mod cells;
mod body;
mod engine;
mod grid;
mod world;

pub use crate::{
  engine::Direction,
  world::World
};

/* Пример добавления вывода в консоль браузера из Rust кода

#[wasm_bindgen]
extern "C" {
  #[wasm_bindgen(js_namespace = console)]
  fn log(s: &str);
}

macro_rules! console_log {
  ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

fn using_a_macro() {
  console_log!("Hello {}!", "world");
  console_log!("Let's print some numbers...");
  console_log!("1 + 3 = {}", 1 + 3);
}
*/