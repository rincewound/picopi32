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
                postion_handler!(self, hero, update_scalar_y, get_scalar_x, get_scalar_y, 0, -1, false, 0, -1);
            },
            Keys::Down =>
            {
                postion_handler!(self, hero, update_scalar_y, get_scalar_x, get_scalar_y, 0, 1, true, 0, 1);
            }
            Keys::Left =>
            {
                postion_handler!(self, hero, update_scalar_x, get_scalar_y, get_scalar_x, -1, 0, false, -1, 0);
            }
            Keys::Right =>
            {
                postion_handler!(self, hero, update_scalar_x, get_scalar_y, get_scalar_x, 1, 0, true, 1, 0);
            }
        };

        self.update_hero_state_if_nedded();
    }

    fn generate_steping_position(&self, x_scal: isize, y_scal: isize) -> Position
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

    pub fn set_monster_in_field(&mut self, moster_id: usize, x: usize, y: usize)
    {
        self.map[y][x] = FieldElements::Monster as usize;
        self.monsters[moster_id].update_position(Position{x: x as isize, y: y as isize, scalar_x: 0, scalar_y: 0});
    }

    fn update_monster_postion(&mut self)
    {
        // TODO: randomize generation of monster postition
    }

    fn update_hero_state_if_nedded(&mut self)
    {
        let hero_pos = self.hero.get_position();
        let monsters= self.monsters.clone();
        for monster in monsters
        {
            let _monster = monster.clone();
            if hero_pos != monster.get_position()
            {
                continue;
            }

            self.hero.game_over();
        }
    }

    pub fn get_entity(&self, x: usize, y: usize) -> usize
    {
        self.map[y][x]
    }
}

