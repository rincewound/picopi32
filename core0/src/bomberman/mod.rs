/*
MapNavigatorTrait object should eventually able to communicate with the graphic
core to update the view on the screen.
TODO: this is not specified yet


Entity:
* Movable: those entities 1) are the boxes, which could be destroyed by a bomb.
                          2) are items to increase bomb range, etc..
                          3) could als be the key or door to win a game
* Hero: the main player entity (there could be multiple heros in the future, but for now, let's keep it simple)
* Enemy: the ugly beast which can kill u
* Wall: is simply a wall with the shape of box, that could not be destroyed or. damaged
* FreeField: the way to walk around
*/

use crate::common::Keys;

pub mod utils;
use utils::{EntityType, Actions, Position};

pub mod map_navigator;
use map_navigator::MapNavigatorTrait;


pub mod entities;
use entities::{HeroEntity, Entity};

