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
