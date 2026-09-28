pub mod image;
pub mod rectangle;

#[derive(Clone, Copy)]
pub struct Rect<T> {
    position: [T; 2],
    size: [T; 2],
}

impl<T: Copy> Rect<T> {
    pub const fn new(x: T, y: T, width: T, height: T) -> Self {
        Self {
            position: [x, y],
            size: [width, height],
        }
    }

    pub fn x(&self) -> T {
        self.position[0]
    }
    pub fn y(&self) -> T {
        self.position[1]
    }
    pub fn width(&self) -> T {
        self.size[0]
    }
    pub fn height(&self) -> T {
        self.size[1]
    }
}

#[derive(Clone, Copy)]
pub struct Color {
    red: f32,
    green: f32,
    blue: f32,
    alpha: f32,
}

impl Color {
    pub const fn new(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }
}
