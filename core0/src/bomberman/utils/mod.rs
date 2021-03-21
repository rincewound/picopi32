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


#[derive(Clone, Copy)]
pub struct Position
{
    pub x: u8,
    pub y: u8,
}


