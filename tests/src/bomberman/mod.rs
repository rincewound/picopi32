#[cfg(test)]
mod bomberman_tests
{
    use std::panic;
    use core0::
    {
        bomberman::
        {
            map_navigator::{MapNavigator, FieldElements},
            utils::{Position, Actions, PIXEL_COUNT_PER_ROW},
        },
        common::Keys,
    };

    fn run_test<T>(test: T) -> ()
        where T: FnOnce() -> () + panic::UnwindSafe
    {
        generate_filed();
        let result = panic::catch_unwind(|| {
            test()

        });
        teardown();
        assert!(result.is_ok())
    }

    fn generate_filed() -> MapNavigator
    {
        let mut map_navigator = MapNavigator::new(8);
        map_navigator.set_wall();
        map_navigator.set_player_position();
        map_navigator
    }

    fn teardown()
    {

    }

    #[test]
    fn test_map_navigator_set_filed_wall_elements()
    {
        let mut map_navigator = generate_filed();
        for y in 0..map_navigator.map.len()
        {
            for x in 0..map_navigator.map[0].len()
            {
                if (y % 2 == 1) && (x % 2 == 1)
                {
                    assert_eq!(map_navigator.get_entity(y, y), FieldElements::Wall as usize);
                }
            }
        }
    }


    #[test]
    fn test_set_player_initial_position()
    {
        let map_navigator = generate_filed();
        assert_eq!(map_navigator.get_entity(0, 0), FieldElements::Player as usize);
    }

    #[test]
    fn test_do_action()
    {

    }

    #[test]
    fn test_move_player()
    {
        let mut map_navigator = generate_filed();

        assert_move(&mut map_navigator, Keys::Down, 0, 1);
        assert_move(&mut map_navigator, Keys::Up, 0, 0);
        assert_move(&mut map_navigator, Keys::Right, 1, 0);
        assert_move(&mut map_navigator, Keys::Left, 0, 0);
    }

    fn assert_move(map_navigator: &mut MapNavigator, key: Keys, x: usize, y: usize)
    {
        for _ in 0..PIXEL_COUNT_PER_ROW
        {
            map_navigator.handle_key(key);
        }

        assert_eq!(map_navigator.get_entity(x, y), FieldElements::Player as usize);
    }

    #[test]
    fn test_change_direction_while_player_between_two_rocks()
    {
        let mut map_navigator = generate_filed();
        assert_move(&mut map_navigator, Keys::Down, 0, 1);
        assert_move(&mut map_navigator, Keys::Right, 0, 1);
        assert_move(&mut map_navigator, Keys::Down, 0, 2);
        map_navigator.handle_key(Keys::Right);
        map_navigator.handle_key(Keys::Right);
        // steping down is not possible any mode
        // player already takes the path to the right
        assert_move(&mut map_navigator, Keys::Down, 0, 2);

        map_navigator.handle_key(Keys::Left);
        map_navigator.handle_key(Keys::Left);
        // now its possible 
        assert_move(&mut map_navigator, Keys::Down, 0, 3);

        map_navigator.handle_key(Keys::Left);
        assert_move(&mut map_navigator, Keys::Down, 0, 4);
        assert_move(&mut map_navigator, Keys::Up, 0, 3);
        assert_move(&mut map_navigator, Keys::Up, 0, 2);
        assert_move(&mut map_navigator, Keys::Right, 1, 2);
        assert_move(&mut map_navigator, Keys::Right, 2, 2);

        map_navigator.handle_key(Keys::Down);
        // player make a move to one direction, not able any more
        // to change to another direction
        assert_move(&mut map_navigator, Keys::Right, 2, 2);
        // revert move
        map_navigator.handle_key(Keys::Up);
        assert_move(&mut map_navigator, Keys::Right, 3, 2);

        // player between two rocks
        assert_move(&mut map_navigator, Keys::Up, 3, 2);
        assert_move(&mut map_navigator, Keys::Down, 3, 2);

        // leave rocks
        assert_move(&mut map_navigator, Keys::Right, 4, 2);
        assert_move(&mut map_navigator, Keys::Up, 4, 1);
        assert_move(&mut map_navigator, Keys::Down, 4, 2);

        map_navigator.handle_key(Keys::Up);
        assert_move(&mut map_navigator, Keys::Right, 4, 1);
        map_navigator.handle_key(Keys::Down);
        assert_move(&mut map_navigator, Keys::Right, 5, 2);
    }

    #[test]
    fn test_gameover()
    {
        let mut map_navigator = generate_filed();
        map_navigator.set_monster_in_field(0, 1, 0);
        assert_move(&mut map_navigator, Keys::Right, 1, 0);
        assert_eq!(map_navigator.hero.is_alive(), false);
    }
}
