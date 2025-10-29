pub mod event;
pub mod theme;
pub mod widget;

#[derive(Debug, Clone)]
pub struct Coordinate {
    pub x: u16,
    pub y: u16,
}
impl Coordinate {
    pub fn new(x: u16, y: u16) -> Self {
        Coordinate { x, y }
    }
}
