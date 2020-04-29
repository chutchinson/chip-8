use coffee::graphics::{Frame, Color, Shape, Rectangle, Mesh};

pub struct Gpu {
    width: u8,
    height: u8,
    vram: [u8; 4096]
}

impl Gpu {

    pub fn new() -> Self {
        Gpu {
            width: 64,
            height: 32,
            vram: [0; 4096]
        }
    }

    pub fn draw_sprite(&mut self, addr: u16, len: u16, x: u16, y: u16) {
        log!("[gpu] sprite");
    }

    pub fn reset(&mut self) {
        log!("[gpu] reset");
    }

    pub fn render(&mut self, frame: &mut Frame) {
        // println!("[gpu] render");
        frame.clear(Color::BLACK);

        // let mut mesh = Mesh::new();

        // mesh.fill(
        //     Shape::Rectangle(Rectangle {
        //         x: 0.0,
        //         y: 0.0,
        //         width: 200.0,
        //         height: 200.0
        //     }), Color::WHITE);
        // mesh.draw(&mut frame.as_target());
    }

}