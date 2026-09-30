use crate::renderer::{DestinationRect, Rect};

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

    pub fn dst_x(&self, viewport_size: [u32; 2]) -> f32 {
        self.destination.rect.to_ndc(viewport_size)[0]
    }
    pub fn dst_y(&self, viewport_size: [u32; 2]) -> f32 {
        self.destination.rect.to_ndc(viewport_size)[1]
    }
    pub fn dst_width(&self, viewport_size: [u32; 2]) -> f32 {
        self.destination.rect.to_ndc(viewport_size)[2]
    }
    pub fn dst_height(&self, viewport_size: [u32; 2]) -> f32 {
        self.destination.rect.to_ndc(viewport_size)[3]
    }
    pub fn opacity(&self) -> f32 {
        self.destination.opacity
    }

    pub fn vertices(
        &self,
        texture_size: [u32; 2],
        viewport_size: [u32; 2],
    ) -> [ImageVertex; IMAGE_VERTEX_COUNT] {
        let source_left = self.src_x() as f32 / texture_size[0] as f32;
        let source_top = self.src_y() as f32 / texture_size[1] as f32;
        let source_right = (self.src_x() + self.src_width()) as f32 / texture_size[0] as f32;
        let source_bottom = (self.src_y() + self.src_height()) as f32 / texture_size[1] as f32;
        let left = self.dst_x(viewport_size);
        let top = self.dst_y(viewport_size);
        let width = self.dst_width(viewport_size);
        let height = self.dst_height(viewport_size);
        let right = left + width;
        let bottom = top - height;
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
    path: &'a str,
    source: Rect<u32>,
}
impl<'a> ImageSource<'a> {
    pub const fn new(path: &'a str, source: Rect<u32>) -> Self {
        Self { path, source }
    }

    pub fn x(&self) -> u32 {
        self.source.x()
    }
    pub fn y(&self) -> u32 {
        self.source.y()
    }
    pub fn width(&self) -> u32 {
        self.source.width()
    }
    pub fn height(&self) -> u32 {
        self.source.height()
    }
}

pub struct ImageDestination {
    rect: DestinationRect,
    opacity: f32,
}
impl ImageDestination {
    pub fn new(rect: impl Into<DestinationRect>, opacity: f32) -> Self {
        Self {
            rect: rect.into(),
            opacity,
        }
    }
}

pub struct ImageTexture {
    pub bind_group: wgpu::BindGroup,
    pub size: [u32; 2],
}

pub fn load_image_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    bind_group_layout: &wgpu::BindGroupLayout,
    path: &str,
) -> ImageTexture {
    let image = image::open(path)
        .unwrap_or_else(|error| panic!("画像の読み込みに失敗しました ({path}): {error}"))
        .to_rgba8();
    let size = [image.width(), image.height()];
    let image_size = wgpu::Extent3d {
        width: size[0],
        height: size[1],
        depth_or_array_layers: 1,
    };
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("image texture"),
        size: image_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &image,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * size[0]),
            rows_per_image: Some(size[1]),
        },
        image_size,
    );
    let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("image sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    });
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("image texture bind group"),
        layout: bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&texture_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    });

    ImageTexture { bind_group, size }
}
