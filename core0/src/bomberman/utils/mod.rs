#![no_std]

use std::ops;
use crate::check_valid_scalar;


pub const PIXEL_COUNT_PER_ROW: isize = 10;
pub const DISPLAY_WIDTH: usize = 320;
pub const DISPLAY_HEIGHt: usize = 240;


pub enum EntityType
{
    FieldFree = 0,
    Hero,
    Enemy,
    Movable,
    Wall,
}

pub enum Actions
{
    PlantBomb,
    BreakStone,
    CollectKey, // TODO: do we need this ?
}

pub struct Bomb
{
    range: u8,
}

impl Bomb
{
    pub fn new(range: u8) -> Self
    {
        Self{range}
    }
}


#[derive(Clone, Copy, Default, Debug, PartialEq, PartialOrd)]
pub struct Position
{
    pub x: isize,
    pub y: isize,
    pub scalar_x: isize,
    pub scalar_y: isize,
}


impl ops::Add<Position> for Position
{
    type Output = Position;

    fn add(self, rhs: Position) -> Position
    {
        check_valid_scalar!(self, rhs, scalar_x, scalar_y);
        check_valid_scalar!(self, rhs, scalar_y, scalar_x);

        let mut pos = Position{x: self.x + rhs.x, y: self.y + rhs.y, scalar_x: self.scalar_x + rhs.scalar_x, scalar_y: self.scalar_y + rhs.scalar_y };
        if pos.scalar_x > PIXEL_COUNT_PER_ROW - 1
        {
            pos.scalar_x = 0;
            pos.x += 1;
        }

        if pos.scalar_y > PIXEL_COUNT_PER_ROW - 1
        {
            pos.scalar_y = 0;
            pos.y += 1;
        }

        if pos.scalar_x < 0
        {
            pos.scalar_x = PIXEL_COUNT_PER_ROW - 1;
            pos.x -= 1;
        }

        if pos.scalar_y < 0
        {
            pos.scalar_y = PIXEL_COUNT_PER_ROW - 1;
            pos.y -= 1;
        }

        pos
    }
}

impl Position
{
    pub fn new() -> Self
    {
        Self {x: 0, y: 0, scalar_x: 0, scalar_y: 0}
    }

    pub fn update_pos(&mut self, x: isize, y: isize)
    {
        self.x = x;
        self.y = y;
    }

    pub fn reset(&mut self)
    {
        self.x = 0;
        self.y = 0;
    }
}


#[macro_export]
macro_rules! postion_handler {
    ($self: ident,
     $entity: ident,
     $update_scalar: ident,
     $scalar_A: ident,
     $scalar_B: ident,
     $x: expr,
     $y: expr,
     $is_pos: expr,
     $scal_x: expr,
     $scal_y: expr) =>
    {
        let next_pos = $self.$entity.get_position() + $self.generate_steping_position($scal_x, $scal_y);
        
        if !$self.is_valid_position(next_pos)
        {
            $self.$entity.reinit_scalars();
            return
        }
        $self.$entity.$update_scalar($is_pos);
        
        $self.map[$self.$entity.get_position().y as usize][$self.$entity.get_position().x as usize] = FieldElements::EmptyField as usize;
        $self.$entity.update_position(next_pos);
        $self.map[$self.$entity.get_position().y as usize][$self.$entity.get_position().x as usize] = FieldElements::Player as usize;

    };
}

#[macro_export]
macro_rules! inlineif {
    ($cond: expr,
     $stat1: expr,
     $stat2: expr) =>
    {
        if $cond {
            $stat1 
        } else
        {
            $stat2
        }
    };
}

#[macro_export]
macro_rules! check_valid_scalar {
    ($self: ident,
     $rhs: ident,
     $scalar_A: ident,
     $scalar_B: ident) =>
    {
        if $rhs.$scalar_A != 0
        {
            if $self.$scalar_B != 0
            {
                return $self
            }
        }
    };
}
