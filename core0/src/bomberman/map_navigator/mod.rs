use crate::common::Keys;
use crate::bomberman::utils::{EntityType, Actions, Position};


pub trait MapNavigatorTrait
{
    fn update_and_return_new_pos_if_possible(&self, current_pos: Position, key: Keys) -> Option<Position>;
    fn handle_action(&self, current_pos: Position, action: Actions);
}

pub struct MapNavigator
{
    map: [[i32; 100]; 100],
}

impl MapNavigator
{
    pub fn new() -> Self
    {
        Self {map: [[0; 100]; 100]}
    }
}

impl MapNavigatorTrait for MapNavigator
{
    fn update_and_return_new_pos_if_possible(&self, current_pos: Position, key: Keys) -> Option<Position>
    {
        Some(Position{x:1, y:2})
    }

    fn handle_action(&self, current_pos: Position, Action: Actions)
    {
    }
}
