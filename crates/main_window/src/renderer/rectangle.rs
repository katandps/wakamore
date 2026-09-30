use crate::renderer::{Color, DestinationRect};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RectangleVertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
}

pub const RECTANGLE_VERTEX_COUNT: usize = 6;

pub struct RectangleDrawOptions {
    pub destination: DestinationRect,
    pub color: Color,
}

impl RectangleDrawOptions {
    pub fn new(destination: impl Into<DestinationRect>, color: Color) -> Self {
        Self {
            destination: destination.into(),
            color,
        }
    }

    pub fn vertices(&self, viewport_size: [u32; 2]) -> [RectangleVertex; RECTANGLE_VERTEX_COUNT] {
        let [left, top, width, height] = self.destination.to_ndc(viewport_size);
        let right = left + width;
        let bottom = top - height;
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
