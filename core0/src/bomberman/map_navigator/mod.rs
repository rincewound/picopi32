#![no_std]

use heapless::{
    Vec,
    consts::U8,
};


use crate::common::Keys;
use crate::bomberman::utils::{
    Actions,
    Position,
    PIXEL_COUNT_PER_ROW,
    DISPLAY_WIDTH,
    DISPLAY_HEIGHt,
};

use crate::postion_handler;

use crate::bomberman::entities::MovableEntity;



#[derive(Clone, Copy)]
pub enum FieldElements
{
    EmptyField = 0,
    Wall = 1,
    Box = 2,
    Player = 3,
    Monster = 4,
}


const ARRAY_WIDTH: usize = DISPLAY_WIDTH / PIXEL_COUNT_PER_ROW as usize;
const ARRAY_HEIGHT: usize = DISPLAY_HEIGHt / PIXEL_COUNT_PER_ROW as usize;
const MAX_MONSTER_COUNT: u8 = 8;
fn get_count(num: u8) -> u8
{
    if num > MAX_MONSTER_COUNT
    {
        return MAX_MONSTER_COUNT
    }

    return num
}


pub struct MapNavigator
{
    pub map: [[usize; ARRAY_WIDTH]; ARRAY_HEIGHT],
    pub hero: MovableEntity,
    pub monsters: Vec<MovableEntity, U8>,
}

impl MapNavigator
{
    pub fn new(num_monsters: u8) -> Self
    {
        let mut monsters = Vec::new();

        for _ in 0..get_count(num_monsters) {
            monsters.push(MovableEntity::new());
        }

        Self {map: [[0; ARRAY_WIDTH]; ARRAY_HEIGHT], hero: MovableEntity::new(), monsters}
    }

    fn setup_field(&mut self)
    {
        // maybe move this to the interface
        // place component inside the game field
        // * update player and monster position
        // * random positioning of breakable rocks
        // * etc ..
    }

    pub fn set_wall(&mut self)
    {
        // assume that each element inside the map is PIXEL_COUNT_PER_ROW x PIXEL_COUNT_PER_ROW block
        let mut start_pos_y = 0;
        loop
        {
            if start_pos_y >= ARRAY_HEIGHT
            {
                break;
            }

            start_pos_y += 1;

            if start_pos_y % 2 == 0
            {
                continue;
            }

            self.set_block(start_pos_y);
        }
        // TODO: remove debug code
        // for index in 0..ARRAY_HEIGHT
        // {
        //     println!("{:?}", self.map[index]);
        // }
    }

    fn set_block(&mut self, start_pos_y: usize)
    {
        let mut start_pos_x = 0;
        // populate blocks
        for index in 0..ARRAY_WIDTH
        {
            start_pos_x += 1;
            if start_pos_x >= ARRAY_WIDTH
            {
                break;
            }

            if start_pos_x % 2 == 0
            {
                continue;
            }

            self.map[start_pos_y][start_pos_x] = FieldElements::Wall as usize;
        }
    }

    pub fn set_player_position(&mut self)
    {
        self.map[0][0] = FieldElements::Player as usize;
    }

    fn set_box(&mut self)
    {
    }

    pub fn handle_key(&mut self, key: Keys)
    {
        match key {
            Keys::Up =>
            {
                postion_handler!(self, update_scalar_y, get_scalar_x, get_scalar_y, 0, -1, false, 0, -1);
            },
            Keys::Down =>
            {
                postion_handler!(self, update_scalar_y, get_scalar_x, get_scalar_y, 0, 1, true, 0, 1);
            }
            Keys::Left =>
            {
                postion_handler!(self, update_scalar_x, get_scalar_y, get_scalar_x, -1, 0, false, -1, 0);
            }
            Keys::Right =>
            {
                postion_handler!(self, update_scalar_x, get_scalar_y, get_scalar_x, 1, 0, true, 1, 0);
            }
        };
    }

    fn generate_position(&self, x: isize, y: isize, x_scal: isize, y_scal: isize) -> Position
    {
        Position{x: 0, y: 0, scalar_x: x_scal, scalar_y: y_scal}
    }

    fn is_valid_position(&self, position: Position) -> bool
    {
        if position.x < 0 || position.y < 0
        {
            return false
        }

        self.map[position.y as usize][position.x as usize] != FieldElements::Wall as usize
    }

    fn handle_action(&self, action: Actions)
    {
        match action
        {
            Actions::PlantBomb => (),
            _ => ()
        }
    }

    fn update_monster_postion(&mut self)
    {
    }

    pub fn get_entity(&self, x: usize, y: usize) -> usize
    {
        self.map[y][x]
    }
}

// pub struct EntitiesManager<N, H> 
// where N: MapNavigatorTrait, H: Entity
// {
//     map_navigator: N,
//     hero: H,
//     num_of_monsters: u8,
// }

// impl<N: MapNavigatorTrait, H: Entity> EntitiesManager<N, H>
// {
//     pub fn new(map_navigator: N, hero: H, num_of_monsters: u8) -> Self
//     {
//         Self {map_navigator, hero, num_of_monsters}
//     }

//     pub fn move_player(&mut self, key: Keys)
//     {
//         let current_pos = self.hero.get_position();
//         let new_pos = self.map_navigator.update_and_return_new_pos_if_possible(current_pos, key);
//         self.hero.update_new_position_if_possible(new_pos);
//     }

//     pub fn do_action(&self, action: Actions)
//     {
//         let current_pos = self.hero.get_position();
//         self.map_navigator.handle_action(current_pos, action);
//     }
// }

