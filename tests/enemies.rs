use oddseed_until_dawn::game::{Controls, Difficulty, Game, Phase};

fn move_to(game: &mut Game, returning: bool, sprint: bool) {
    for _ in 0..300 {
        if if returning {
            game.near_station()
        } else {
            game.near_fuel()
        } {
            return;
        }
        game.advance_with_controls(
            0.05,
            Controls {
                direction: if returning { -1 } else { 1 },
                sprint,
            },
        );
        assert_eq!(game.phase, Phase::Playing, "{game:?}");
    }
    panic!("Destination not reached: {game:?}");
}

fn pickup(game: &mut Game) {
    assert!(game.search());
    move_to(game, false, false);
    assert!(game.interact());
}

fn hunted() -> Game {
    let mut game = Game::new(Difficulty::Nightmare);
    game.start();
    pickup(&mut game);
    game
}

#[test]
fn pursuers_appear_only_after_a_nightmare_pickup() {
    for difficulty in Difficulty::ALL {
        let mut game = Game::new(difficulty);
        game.start();
        assert!(game.enemies.is_empty());
        game.search();
        move_to(&mut game, false, false);
        assert!(game.enemies.is_empty());
        game.interact();
        assert_eq!(
            game.enemies.len(),
            if difficulty == Difficulty::Nightmare {
                2
            } else {
                0
            }
        );
    }
}

#[test]
fn pursuing_enemies_give_a_warning_and_then_follow_the_operator() {
    let mut game = hunted();
    let initial = game.enemies.clone();
    assert!(initial
        .iter()
        .all(|enemy| enemy.position > game.position && enemy.wake_in > 0.0));
    game.advance(0.4);
    assert_eq!(game.enemies[0].position, initial[0].position);
    game.advance_with_controls(
        0.6,
        Controls {
            direction: -1,
            sprint: true,
        },
    );
    assert!(game.enemies[0].position < initial[0].position);
    assert_eq!(game.enemies[0].wake_in, 0.0);
    assert!(game.enemies[1].wake_in > 0.0);
}

#[test]
fn waiting_in_front_of_a_pursuer_ends_the_run() {
    let mut game = hunted();
    let fuel = game.fuel;
    game.advance(3.0);
    assert_eq!(game.phase, Phase::Caught);
    assert!(game.is_finished());
    assert!(game.fuel <= fuel);
    assert_eq!(game.searches_left, 3);
    assert!(game.carrying);
}

#[test]
fn sprinting_home_escapes_and_clears_the_pursuers() {
    let mut game = hunted();
    move_to(&mut game, true, true);
    let fuel = game.fuel;
    assert!(game.interact());
    assert!(game.enemies.is_empty());
    assert_eq!(game.nearest_enemy_distance(), None);
    assert!(!game.outside());
    assert_eq!(game.searches_left, 2);
    assert_eq!(game.fuel, fuel + game.difficulty.fuel_reward());
    game.advance(3.0);
    assert_eq!(game.phase, Phase::Playing);
    assert!(game.enemies.is_empty());
}

#[test]
fn walking_home_from_the_second_cache_is_too_slow() {
    let mut game = hunted();
    move_to(&mut game, true, true);
    game.interact();
    game.toggle(1);
    game.advance(18.1);
    pickup(&mut game);
    game.advance_with_controls(
        10.0,
        Controls {
            direction: -1,
            sprint: false,
        },
    );
    assert_eq!(game.phase, Phase::Caught);
    assert!(!game.near_station());
}

#[test]
fn pause_freezes_pursuers_and_their_warning_timers() {
    let mut game = hunted();
    game.advance(0.2);
    game.pause();
    let enemies = game.enemies.clone();
    let before = (game.elapsed, game.position, game.searching);
    game.advance_with_controls(
        10.0,
        Controls {
            direction: -1,
            sprint: true,
        },
    );
    assert_eq!(game.enemies, enemies);
    assert_eq!(before, (game.elapsed, game.position, game.searching));
    assert!(!game.select_difficulty(Difficulty::Night));
    game.pause();
    game.advance_with_controls(
        1.0,
        Controls {
            direction: -1,
            sprint: true,
        },
    );
    assert_ne!(game.enemies[0].position, enemies[0].position);
}

#[test]
fn restarting_removes_pursuers_from_the_previous_run() {
    let mut game = hunted();
    game.start();
    assert!(game.enemies.is_empty());
    assert!(!game.carrying);
    assert!(!game.outside());
    assert_eq!(game.difficulty, Difficulty::Nightmare);
}

#[test]
fn caught_games_do_not_keep_chasing_or_accept_actions() {
    let mut game = hunted();
    game.advance(3.0);
    let enemies = game.enemies.clone();
    let before = (game.elapsed, game.position, game.fuel);
    game.advance_with_controls(
        20.0,
        Controls {
            direction: -1,
            sprint: true,
        },
    );
    assert_eq!(game.enemies, enemies);
    assert_eq!(before, (game.elapsed, game.position, game.fuel));
    assert!(!game.interact());
    assert!(!game.toggle(0));
    assert!(!game.repair());
    game.pause();
    assert_eq!(game.phase, Phase::Caught);
    assert!(game.select_difficulty(Difficulty::Night));
    assert!(game.enemies.is_empty());
}

#[test]
fn chase_updates_are_consistent_across_frame_sizes() {
    let mut whole = hunted();
    let mut frames = whole.clone();
    let controls = Controls {
        direction: -1,
        sprint: true,
    };
    whole.advance_with_controls(1.0, controls);
    for _ in 0..20 {
        frames.advance_with_controls(0.05, controls);
    }
    assert!((whole.position - frames.position).abs() < 0.0001);
    for (first, second) in whole.enemies.iter().zip(&frames.enemies) {
        assert!((first.position - second.position).abs() < 0.0001);
        assert!((first.wake_in - second.wake_in).abs() < 0.0001);
    }
}

#[test]
fn running_into_an_active_enemy_cannot_pass_through_it() {
    let mut game = hunted();
    game.enemies[0].wake_in = 0.0;
    game.enemies[0].position = game.position + 0.04;
    game.advance_with_controls(
        0.1,
        Controls {
            direction: 1,
            sprint: true,
        },
    );
    assert_eq!(game.phase, Phase::Caught);
}
