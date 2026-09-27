use crate::renderer::Rect;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ImageVertex {
    pub position: [f32; 2],
    pub texture_coordinates: [f32; 2],
    pub opacity: f32,
}

pub const IMAGE_VERTEX_COUNT: usize = 6;

pub struct ImageDrawOptions<'a> {
    source: ImageSource<'a>,
    destination: ImageDestination,
}

impl<'a> ImageDrawOptions<'a> {
    pub fn new(source: ImageSource<'a>, destination: ImageDestination) -> Self {
        Self {
            source,
            destination,
        }
    }

    pub fn src_path(&self) -> &str {
        self.source.path
    }
    pub fn src_x(&self) -> u32 {
        self.source.x()
    }
    pub fn src_y(&self) -> u32 {
        self.source.y()
    }
    pub fn src_width(&self) -> u32 {
        self.source.width()
    }
    pub fn src_height(&self) -> u32 {
        self.source.height()
    }

    pub fn dst_x(&self) -> f32 {
        self.destination.rect.position[0]
    }
    pub fn dst_y(&self) -> f32 {
        self.destination.rect.position[1]
    }
    pub fn dst_width(&self) -> f32 {
        self.destination.rect.size[0]
    }
    pub fn dst_height(&self) -> f32 {
        self.destination.rect.size[1]
    }
    pub fn opacity(&self) -> f32 {
        self.destination.opacity
    }

    pub fn vertices(&self, texture_size: [u32; 2]) -> [ImageVertex; IMAGE_VERTEX_COUNT] {
        let source_left = self.src_x() as f32 / texture_size[0] as f32;
        let source_top = self.src_y() as f32 / texture_size[1] as f32;
        let source_right = (self.src_x() + self.src_width()) as f32 / texture_size[0] as f32;
        let source_bottom = (self.src_y() + self.src_height()) as f32 / texture_size[1] as f32;
        let left = self.dst_x();
        let top = self.dst_y();
        let right = left + self.dst_width();
        let bottom = top - self.dst_height();
        let opacity = self.opacity().clamp(0.0, 1.0);

        [
            ImageVertex {
                position: [left, top],
                texture_coordinates: [source_left, source_top],
                opacity,
            },
            ImageVertex {
                position: [right, top],
                texture_coordinates: [source_right, source_top],
                opacity,
            },
            ImageVertex {
                position: [left, bottom],
                texture_coordinates: [source_left, source_bottom],
                opacity,
            },
            ImageVertex {
                position: [right, top],
                texture_coordinates: [source_right, source_top],
                opacity,
            },
            ImageVertex {
                position: [right, bottom],
                texture_coordinates: [source_right, source_bottom],
                opacity,
            },
            ImageVertex {
                position: [left, bottom],
                texture_coordinates: [source_left, source_bottom],
                opacity,
            },
        ]
    }
}

pub struct ImageSource<'a> {
    pub path: &'a str,
    pub source: Rect<u32>,
}
impl<'a> ImageSource<'a> {
    pub fn x(&self) -> u32 {
        self.source.position[0]
    }
    pub fn y(&self) -> u32 {
        self.source.position[1]
    }
    pub fn width(&self) -> u32 {
        self.source.size[0]
    }
    pub fn height(&self) -> u32 {
        self.source.size[1]
    }
}

pub struct ImageDestination {
    pub rect: Rect<f32>,
    pub opacity: f32,
}
impl ImageDestination {
    pub fn x(&self) -> f32 {
        self.rect.position[0]
    }
    pub fn y(&self) -> f32 {
        self.rect.position[1]
    }
    pub fn width(&self) -> f32 {
        self.rect.size[0]
    }
    pub fn height(&self) -> f32 {
        self.rect.size[1]
    }
}
