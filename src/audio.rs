use sdl2::Sdl;

pub struct Audio {
    pub delay: u8,
    pub sound: u8,
}

impl Audio {
    pub fn new(_sdl: &Sdl) -> Self {
        Audio { delay: 0, sound: 0 }
    }

    pub fn tick(&mut self) {
        if self.delay > 0 {
            self.delay -= 1
        }

        if self.sound > 0 {
            self.delay -= 1
        }
    }
}
