use crate::renderer::{Color, Rect};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RectangleVertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
}

pub const RECTANGLE_VERTEX_COUNT: usize = 6;

pub struct RectangleDrawOptions {
    pub destination: Rect<f32>,
    pub color: Color,
}

impl RectangleDrawOptions {
    pub fn new(destination: Rect<f32>, color: Color) -> Self {
        Self { destination, color }
    }

    pub fn vertices(&self) -> [RectangleVertex; RECTANGLE_VERTEX_COUNT] {
        let left = self.destination.position[0];
        let top = self.destination.position[1];
        let right = left + self.destination.size[0];
        let bottom = top - self.destination.size[1];
        let color = [
            self.color.red,
            self.color.green,
            self.color.blue,
            self.color.alpha.clamp(0.0, 1.0),
        ];

        [
            RectangleVertex {
                position: [left, top],
                color,
            },
            RectangleVertex {
                position: [right, top],
                color,
            },
            RectangleVertex {
                position: [left, bottom],
                color,
            },
            RectangleVertex {
                position: [right, top],
                color,
            },
            RectangleVertex {
                position: [right, bottom],
                color,
            },
            RectangleVertex {
                position: [left, bottom],
                color,
            },
        ]
    }
}
