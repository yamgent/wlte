#[derive(Debug, Clone, Copy)]
pub struct Position<T> {
    pub x: T,
    pub y: T,
}

#[derive(Debug, Clone, Copy)]
pub struct Size<T> {
    pub w: T,
    pub h: T,
}

#[derive(Debug, Clone, Copy)]
pub struct Bounds<T> {
    // by convention, the top left position
    pub pos: Position<T>,
    pub size: Size<T>,
}

impl<T: std::ops::Add<Output = T> + Clone + Copy> Bounds<T> {
    pub fn right(&self) -> T {
        self.pos.x + self.size.w
    }

    pub fn left(&self) -> T {
        self.pos.x
    }

    pub fn top(&self) -> T {
        self.pos.y
    }

    pub fn bottom(&self) -> T {
        self.pos.y + self.size.h
    }
}
