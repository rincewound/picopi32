use crate::bomberman::utils::{Position, PIXEL_COUNT_PER_ROW};
use crate::inlineif;



pub trait Monster
{
    fn is_alive(&self) -> bool;
    fn update_new_position_if_possible(&mut self, current_pos: Option<Position>);
    fn get_position(&self) -> Position;
    fn game_over(&mut self);
}

// MovableEtity could be in this case the Hero or any of the Monster
pub struct MovableEntity
{
    position: Position,
    is_alive: bool,
}

impl MovableEntity
{
    pub fn new() -> Self
    {
        Self{ position: Position::new(), is_alive: true }
    }
}

impl MovableEntity
{
    pub fn update_position(&mut self, new_pos: Position)
    {
        self.position = new_pos
    }

    pub fn get_position(&self) -> Position
    {
        self.position
    }

    pub fn update_scalar_y(&mut self, is_pos: bool)
    {
        inlineif!(is_pos, self.position.scalar_y += 1, self.position.scalar_x -= 1);
    }

    pub fn update_scalar_x(&mut self, is_pos: bool)
    {
        inlineif!(is_pos, self.position.scalar_x += 1, self.position.scalar_x -= 1);
    }

    pub fn get_scalar_x(&self) -> isize
    {
        self.position.scalar_x.abs()
    }

    pub fn get_scalar_y(&self) -> isize
    {
        self.position.scalar_y.abs()
    }

    pub fn reinit_scalars(&mut self)
    {
        self.position.scalar_x = 0;
        self.position.scalar_y = 0;
    }

    pub fn game_over(&mut self) {
        self.is_alive = false;
        self.position.reset()
    }

    pub fn is_alive(&self) -> bool
    {
        self.is_alive
    }
}
