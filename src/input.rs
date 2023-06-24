use std::collections::HashMap;

use once_cell::sync::Lazy;
use sdl2::{event::Event, keyboard::Scancode};

/// ```text
/// 11 22 33 C4
/// 4Q 5W 6E DR
/// 7A 8S 9D EF
/// AZ 0X BC FV
/// ```
const KEY_MAPPING: Lazy<HashMap<Scancode, usize>> = Lazy::new(|| {
    let mut map: HashMap<Scancode, usize> = HashMap::new();

    map.insert(Scancode::Num1, 0x1);
    map.insert(Scancode::Num2, 0x2);
    map.insert(Scancode::Num3, 0x3);
    map.insert(Scancode::Num4, 0xC);

    map.insert(Scancode::Q, 0x4);
    map.insert(Scancode::W, 0x5);
    map.insert(Scancode::E, 0x6);
    map.insert(Scancode::R, 0xD);

    map.insert(Scancode::A, 0x7);
    map.insert(Scancode::S, 0x8);
    map.insert(Scancode::D, 0x9);
    map.insert(Scancode::F, 0xE);

    map.insert(Scancode::Z, 0xA);
    map.insert(Scancode::X, 0x0);
    map.insert(Scancode::C, 0xB);
    map.insert(Scancode::V, 0xF);

    map
});

pub struct Keypad {
    pub pressed: [bool; 16],
    last_released: Option<u8>,
}

impl Keypad {
    pub fn new() -> Self {
        Keypad {
            pressed: [false; 16],
            last_released: None,
        }
    }

    pub fn listen(&mut self, event: &Event) {
        match event {
            Event::KeyDown { scancode, .. } => self.on_key(scancode, true),
            Event::KeyUp { scancode, .. } => self.on_key(scancode, false),
            _ => {}
        }
    }

    pub fn consume_last_released(&mut self) -> Option<u8> {
        let last_released = self.last_released;
        self.last_released = None;
        last_released
    }

    fn on_key(&mut self, scancode: &Option<Scancode>, down: bool) {
        if let Some(scancode) = scancode {
            if KEY_MAPPING.contains_key(scancode) {
                self.pressed[KEY_MAPPING[scancode]] = down;

                if !down {
                    self.last_released = Some(KEY_MAPPING[scancode] as u8)
                }
            }
        }
    }
}
