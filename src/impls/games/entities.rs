use crate::ui::Coordinate;

pub struct Entity {
    pub x: u16,
    pub y: u16,
    pub display: String,
    pub width: u16,
    pub last_x: u16,
    pub last_y: u16,
}

pub trait GameEntity {
    fn position(&self) -> Coordinate;

    fn last_position(&self) -> Coordinate;

    fn move_to(&mut self, x: u16, y: u16);

    fn display(&self) -> &str;

    fn width(&self) -> u16;

    fn coll_detect(&self, other: &dyn GameEntity) -> bool {
        self.position().x < other.position().x + other.width()
            && self.position().x + self.width() > other.position().x
            && self.position().y == other.position().y
    }
}
