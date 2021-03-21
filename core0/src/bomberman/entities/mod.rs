use crate::common::Keys;
use crate::bomberman::utils::{EntityType, Actions, Position};
use crate::bomberman::map_navigator::MapNavigatorTrait;


const PIXEL_COUNT_PER_ROW: u8 = 8;


pub trait Entity
{
    fn is_alive(&self) -> bool;
    fn update_new_position_if_possible(&mut self, current_pos: Option<Position>);
    fn get_position(&self) -> Position;
    fn game_over(&mut self);
}


pub struct HeroEntityManager<N, H> 
where N: MapNavigatorTrait, H: Entity
{
    map_navigator: N,
    hero: H,
}

impl<N: MapNavigatorTrait, H: Entity> HeroEntityManager<N, H>
{
    pub fn new(map_navigator: N, hero: H) -> Self
    {
        Self {map_navigator: map_navigator, hero: hero}
    }

    pub fn move_player(&mut self, key: Keys)
    {
        let current_pos = self.hero.get_position();
        let new_pos = self.map_navigator.update_and_return_new_pos_if_possible(current_pos, key);
        self.hero.update_new_position_if_possible(new_pos);
    }

    pub fn do_action(&self, action: Actions)
    {
        let current_pos = self.hero.get_position();
        self.map_navigator.handle_action(current_pos, action);
    }
}

pub struct HeroEntity
{
    position: Position,
    is_alive: bool,
}

impl Entity for HeroEntity
{
    fn is_alive(&self) -> bool
    {
        self.is_alive
    }

    fn update_new_position_if_possible(&mut self, new_pos: Option<Position>)
    {
        match new_pos
        {
            Some(pos) => self.position = pos,
            None => return
        }
    }

    fn get_position(&self) -> Position
    {
        self.position
    }

    fn game_over(&mut self) {
        self.is_alive = false
    }
}
