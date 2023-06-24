use sdl2::{pixels::Color, rect::Rect, render::Canvas, video::Window, Sdl};

const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 32;

const SCALE_FACTOR: u32 = 20;
const SCREEN_WIDTH: u32 = DISPLAY_WIDTH as u32 * SCALE_FACTOR;
const SCREEN_HEIGHT: u32 = DISPLAY_HEIGHT as u32 * SCALE_FACTOR;

const COLOR_BG: Color = Color::RGB(0, 0, 0);
const COLOR_FG: Color = Color::RGB(255, 255, 255);

pub const FONT_SET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

pub struct Display {
    ram: [[bool; DISPLAY_WIDTH]; DISPLAY_HEIGHT],
    canvas: Canvas<Window>,
    update: bool,
}

impl Display {
    pub fn new(sdl: &Sdl) -> Self {
        let mut canvas = sdl
            .video()
            .unwrap()
            .window("chip8", SCREEN_WIDTH, SCREEN_HEIGHT)
            .position_centered()
            .opengl()
            .build()
            .unwrap()
            .into_canvas()
            .build()
            .unwrap();

        canvas.set_draw_color(COLOR_BG);
        canvas.clear();
        canvas.present();

        Display {
            ram: [[false; DISPLAY_WIDTH]; DISPLAY_HEIGHT],
            canvas: canvas,
            update: false,
        }
    }

    pub fn draw(&mut self) {
        if !self.update {
            return;
        }

        for (y, row) in self.ram.iter().enumerate() {
            let ry = (y as i32) * (SCALE_FACTOR as i32);

            for (x, &col) in row.iter().enumerate() {
                let rx = (x as i32) * (SCALE_FACTOR as i32);

                if col {
                    self.canvas.set_draw_color(COLOR_FG)
                } else {
                    self.canvas.set_draw_color(COLOR_BG)
                }

                let _ = self
                    .canvas
                    .fill_rect(Rect::new(rx, ry, SCALE_FACTOR, SCALE_FACTOR));
            }
        }

        self.canvas.present();
        self.update = false
    }

    pub fn set(&mut self, x: usize, y: usize, fill: bool) -> bool {
        let cx = x % DISPLAY_WIDTH;
        let cy = y % DISPLAY_HEIGHT;

        let collision = fill && self.ram[cy][cx];
        self.ram[cy][cx] ^= fill;

        self.update = true;
        collision
    }

    pub fn clear(&mut self) {
        for x in 0..DISPLAY_WIDTH {
            for y in 0..DISPLAY_HEIGHT {
                self.ram[y][x] = false;
            }
        }

        self.update = true;
    }
}
