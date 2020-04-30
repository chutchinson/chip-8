use coffee::graphics::{Frame, Color, Shape, Rectangle, Mesh};

pub struct Gpu {
    pub width: usize,
    pub height: usize,
    pub vram: [u8; 4096]
}

impl Gpu {

    pub fn new() -> Self {
        Gpu {
            width: 64,
            height: 32,
            vram: [0; 4096]
        }
    }

    pub fn clear(&mut self) {
        for x in self.vram.iter_mut() {
            *x = 0;
        }
    }

    pub fn draw_sprite(&mut self, 
        memory: &[u8], addr: u16, len: u16, x: u16, y: u16) -> bool {
        let mut collision = false;
        let width = self.width as u16;
        for py in 0..len {
            let pixel = memory[(addr + py) as usize];
            for px in 0..8 {
                if (pixel & (0x80 >> px)) != 0x0 {
                    let addr = x + px + ((y + py) * width);
                    let addr = addr as usize;
                    if self.vram[addr] == 1 {
                        collision |= true;
                    }
                    self.vram[addr] ^= 1;
                }
            }
        }
        collision
    }

    pub fn reset(&mut self) {
        self.clear();
        log!("[gpu] reset");
    }

    pub fn render(&mut self, frame: &mut Frame) {

        frame.clear(Color::BLACK);
        
        let mut mesh = Mesh::new();
        let scale = 10f32;

        for y in 0..self.height {
            for x in 0..self.width {
                let index = y * self.width + x;
                let texel = self.vram[index as usize] & 0x01;
                let x = x as f32;
                let y = y as f32;
                let color = if texel == 0x01 { Color::WHITE } else { Color::BLACK };
                
                mesh.fill(Shape::Rectangle(Rectangle {
                    x: x * scale,
                    y: y * scale,
                    width: scale,
                    height: scale
                }), color);
            }
        }

        mesh.draw(&mut frame.as_target());
    }

}