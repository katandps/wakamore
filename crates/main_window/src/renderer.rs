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
