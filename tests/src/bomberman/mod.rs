#[cfg(test)]
mod bomberman_tests
{
    use core0::
    {
        bomberman::
        {
            entities::{Entity, HeroEntity},
            map_navigator::{MapNavigator, MapNavigatorTrait},
            utils::{Position, Actions},
        },
        common::Keys,
    };

    pub struct FakeMapNavigator;

    impl MapNavigatorTrait for FakeMapNavigator
    {
        fn update_and_return_new_pos_if_possible(&self, current_pos: Position, key: Keys) -> Option<Position> {
            Some(current_pos)
        }

        fn handle_action(&self, current_pos: Position, action: Actions) {}
    }

    #[test]
    fn test_update_player_pos()
    {
    }

    #[test]
    fn test_move_player()
    {
    }

    #[test]
    fn test_do_action()
    {
    }

    #[test]
    fn test_gameover()
    {
    }
}
