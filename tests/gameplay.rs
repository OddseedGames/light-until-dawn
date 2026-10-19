use oddseed_until_dawn::game::{Controls, Difficulty, Game, Phase, NIGHT_SECONDS};

fn started(difficulty: Difficulty) -> Game {
    let mut game = Game::new(difficulty);
    game.start();
    game
}

fn travel(game: &mut Game, returning: bool) {
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
                sprint: true,
            },
        );
        assert_eq!(game.phase, Phase::Playing, "{game:?}");
    }
    panic!("Destination was not reached: {game:?}");
}

fn complete_trip(game: &mut Game) {
    assert!(game.interact());
    travel(game, false);
    assert!(game.interact());
    travel(game, true);
    assert!(game.interact());
}

#[test]
fn ready_games_do_not_consume_resources() {
    let mut game = Game::default();
    game.advance(20.0);
    assert_eq!(game.elapsed, 0.0);
    assert_eq!(game.fuel, game.difficulty.initial_fuel());
    assert!(!game.interact());
    assert!(!game.toggle(1));
}

#[test]
fn pause_freezes_the_operator_and_the_station() {
    let mut game = started(Difficulty::Night);
    game.search();
    game.advance_with_controls(
        1.0,
        Controls {
            direction: 1,
            sprint: true,
        },
    );
    game.pause();
    let before = (
        game.elapsed,
        game.fuel,
        game.position,
        game.searching,
        game.stamina,
        game.threat,
    );
    game.advance_with_controls(
        30.0,
        Controls {
            direction: -1,
            sprint: true,
        },
    );
    assert_eq!(
        before,
        (
            game.elapsed,
            game.fuel,
            game.position,
            game.searching,
            game.stamina,
            game.threat
        )
    );
    assert!(!game.interact());
    game.pause();
    game.advance(0.1);
    assert!(game.elapsed > before.0);
}

#[test]
fn an_unattended_station_freezes() {
    let mut game = started(Difficulty::Night);
    game.advance(NIGHT_SECONDS);
    assert_eq!(game.phase, Phase::Frozen);
}

#[test]
fn fuel_requires_a_pickup_and_a_return_to_the_door() {
    let mut game = started(Difficulty::Night);
    assert!(game.interact());
    assert!(!game.search());
    travel(&mut game, false);
    let before_pickup = game.fuel;
    assert!(game.interact());
    assert!(game.carrying);
    assert_eq!(game.fuel, before_pickup);
    assert_eq!(game.searches_left, 3);
    assert!(!game.interact());
    travel(&mut game, true);
    let before_return = game.fuel;
    assert!(game.interact());
    assert_eq!(game.fuel, before_return + game.difficulty.fuel_reward());
    assert_eq!(game.searches_left, 2);
    assert!(!game.outside());
    assert!(!game.carrying);
    assert!(game.systems[0]);
    assert!(!game.search());
}

#[test]
fn walking_past_a_can_does_not_pick_it_up() {
    let mut game = started(Difficulty::Night);
    game.search();
    travel(&mut game, false);
    game.advance(2.0);
    assert!(!game.carrying);
    assert_eq!(game.searches_left, 3);
    assert!(game.fuel < game.difficulty.initial_fuel());
}

#[test]
fn interaction_is_rejected_between_the_door_and_the_fuel() {
    let mut game = started(Difficulty::Night);
    game.search();
    game.advance_with_controls(
        1.0,
        Controls {
            direction: 1,
            sprint: false,
        },
    );
    assert!(!game.near_station());
    assert!(!game.near_fuel());
    assert!(!game.interact());
}

#[test]
fn an_empty_return_preserves_the_canister() {
    let mut game = started(Difficulty::Night);
    game.search();
    assert!(game.interact());
    assert_eq!(game.searches_left, 3);
    assert_eq!(game.fuel, game.difficulty.initial_fuel());
    assert!(!game.outside());
    assert!(!game.can_search());
    game.advance(6.1);
    assert!(game.can_search());
}

#[test]
fn standing_outside_does_not_automatically_bring_fuel() {
    let mut game = started(Difficulty::Night);
    game.search();
    game.advance(20.0);
    assert_eq!(game.phase, Phase::Lost);
    assert_eq!(game.searches_left, 3);
    assert!(game.fuel < game.difficulty.initial_fuel());
}

#[test]
fn station_controls_and_repairs_are_unavailable_outside() {
    let mut game = started(Difficulty::Night);
    game.integrity = 60.0;
    assert!(game.search());
    assert_eq!(game.systems, [false, false, false]);
    for index in 0..3 {
        assert!(!game.toggle(index));
    }
    assert!(!game.repair());
}

#[test]
fn sprint_trades_stamina_and_noise_for_speed() {
    let mut walking = started(Difficulty::Night);
    walking.search();
    let mut running = walking.clone();
    walking.advance_with_controls(
        3.0,
        Controls {
            direction: 1,
            sprint: false,
        },
    );
    running.advance_with_controls(
        3.0,
        Controls {
            direction: 1,
            sprint: true,
        },
    );
    assert!(running.position > walking.position);
    assert!(running.stamina < walking.stamina);
    assert!(running.threat > walking.threat);
    running.advance(2.0);
    assert!(running.stamina > 22.0);
}

#[test]
fn exhaustion_forces_walking_until_stamina_recovers() {
    let mut game = started(Difficulty::Night);
    game.search();
    game.advance_with_controls(
        2.0,
        Controls {
            direction: 1,
            sprint: true,
        },
    );
    game.advance_with_controls(
        2.0,
        Controls {
            direction: -1,
            sprint: true,
        },
    );
    game.advance_with_controls(
        0.5,
        Controls {
            direction: 1,
            sprint: true,
        },
    );
    assert!(!game.sprinting);
    assert!(game.stamina < 30.0);
    game.advance_with_controls(
        2.0,
        Controls {
            direction: 1,
            sprint: true,
        },
    );
    assert!(game.sprinting);
}

#[test]
fn movement_stays_inside_the_route_and_stamina_cannot_be_negative() {
    let mut game = started(Difficulty::Calm);
    game.search();
    game.advance_with_controls(
        2.0,
        Controls {
            direction: -1,
            sprint: true,
        },
    );
    assert_eq!(game.position, 0.0);
    game.advance_with_controls(
        10.0,
        Controls {
            direction: 1,
            sprint: true,
        },
    );
    assert_eq!(game.position, 1.0);
    assert!((0.0..=100.0).contains(&game.stamina));
}

#[test]
fn there_are_only_three_canisters_and_each_route_gets_longer() {
    let mut game = started(Difficulty::Calm);
    game.toggle(1);
    let mut previous_site = 0.0;
    for remaining in (0..3).rev() {
        assert!(game.fuel_site() > previous_site);
        previous_site = game.fuel_site();
        complete_trip(&mut game);
        assert_eq!(game.searches_left, remaining);
        game.advance(18.1);
    }
    assert!(!game.can_search());
    assert!(!game.search());
}

#[test]
fn difficulty_affects_starting_fuel_cold_consumption_and_exposure() {
    let mut calm = started(Difficulty::Calm);
    let mut hard = started(Difficulty::Nightmare);
    assert!(calm.fuel > hard.fuel);
    assert!(calm.consumption() < hard.consumption());
    calm.advance(10.0);
    hard.advance(10.0);
    assert!(calm.temperature > hard.temperature);
    calm.search();
    hard.search();
    assert!(calm.searching > hard.searching);
    assert!(calm.difficulty.fuel_reward() > hard.difficulty.fuel_reward());
}

#[test]
fn changing_difficulty_is_allowed_only_between_runs() {
    let mut game = Game::default();
    assert!(game.select_difficulty(Difficulty::Nightmare));
    game.start();
    assert!(!game.select_difficulty(Difficulty::Calm));
    game.pause();
    assert!(!game.select_difficulty(Difficulty::Calm));
    game.pause();
    game.advance(NIGHT_SECONDS);
    assert!(game.select_difficulty(Difficulty::Calm));
    assert_eq!(game.phase, Phase::Ready);
    assert_eq!(game.fuel, Difficulty::Calm.initial_fuel());
}

#[test]
fn darkness_eventually_breaks_the_perimeter() {
    let mut game = started(Difficulty::Night);
    game.systems = [false, true, false];
    game.fuel = 100.0;
    game.advance(NIGHT_SECONDS);
    assert_eq!(game.phase, Phase::Overrun);
}

#[test]
fn repair_requires_damage_fuel_and_a_cooldown() {
    let mut game = started(Difficulty::Night);
    assert!(!game.repair());
    game.integrity = 70.0;
    game.fuel = 8.0;
    assert!(!game.repair());
    game.fuel = 12.0;
    assert!(game.repair());
    assert_eq!(game.integrity, 98.0);
    assert_eq!(game.fuel, 3.0);
    game.integrity = 60.0;
    game.fuel = 20.0;
    assert!(!game.repair());
}

#[test]
fn restarting_resets_the_route_and_preserves_difficulty() {
    let mut game = started(Difficulty::Nightmare);
    game.search();
    travel(&mut game, false);
    game.interact();
    game.start();
    assert_eq!(game.phase, Phase::Playing);
    assert_eq!(game.difficulty, Difficulty::Nightmare);
    assert_eq!(game.fuel, Difficulty::Nightmare.initial_fuel());
    assert_eq!(game.position, 0.0);
    assert_eq!(game.elapsed, 0.0);
    assert_eq!(game.searches_left, 3);
    assert!(!game.carrying);
    assert!(!game.outside());
}

#[test]
fn terminal_games_do_not_accept_actions_or_keep_running() {
    let mut game = started(Difficulty::Night);
    game.search();
    game.advance(20.0);
    let before = (game.elapsed, game.fuel, game.position);
    game.advance_with_controls(
        20.0,
        Controls {
            direction: 1,
            sprint: true,
        },
    );
    assert_eq!(before, (game.elapsed, game.fuel, game.position));
    assert!(!game.interact());
    assert!(!game.toggle(1));
    assert!(!game.repair());
}

#[test]
fn simulation_is_consistent_for_small_and_large_updates() {
    let mut whole = started(Difficulty::Night);
    whole.search();
    let mut frames = whole.clone();
    let controls = Controls {
        direction: 1,
        sprint: true,
    };
    whole.advance_with_controls(2.0, controls);
    for _ in 0..40 {
        frames.advance_with_controls(0.05, controls);
    }
    assert!((whole.position - frames.position).abs() < 0.0001);
    assert!((whole.fuel - frames.fuel).abs() < 0.001);
    assert!((whole.searching - frames.searching).abs() < 0.001);
}

#[test]
fn invalid_time_steps_do_not_corrupt_the_simulation() {
    let mut game = started(Difficulty::Night);
    for dt in [f32::NAN, f32::INFINITY, -1.0, 0.0] {
        game.advance(dt);
    }
    assert_eq!(game.elapsed, 0.0);
    assert_eq!(game.fuel, game.difficulty.initial_fuel());
}

fn play_to_end(difficulty: Difficulty, radio: bool) -> Game {
    let mut game = started(difficulty);
    for _ in 0..4000 {
        if game.is_finished() {
            break;
        }
        let mut controls = Controls::default();
        if game.outside() {
            if game.carrying && game.near_station() || !game.carrying && game.near_fuel() {
                game.interact();
            }
            if game.outside() {
                controls.direction = if game.carrying { -1 } else { 1 };
                controls.sprint = if difficulty == Difficulty::Nightmare {
                    game.carrying
                } else {
                    game.stamina > 20.0
                };
            }
        } else {
            if game.temperature < 4.0 && !game.systems[1]
                || game.temperature > 12.0 && game.systems[1]
            {
                game.toggle(1);
            }
            if game.threat > 36.0 && !game.systems[0] || game.threat < 8.0 && game.systems[0] {
                game.toggle(0);
            }
            if game.systems[2] != radio {
                game.toggle(2);
            }
            if game.integrity <= 40.0 && game.can_repair() {
                game.repair();
            }
            if (game.fuel < 24.0 || game.elapsed == 0.0)
                && game.can_search()
                && game.threat < 28.0
                && game.elapsed < NIGHT_SECONDS - 14.0
            {
                game.search();
            }
        }
        game.advance_with_controls(0.05, controls);
    }
    game
}

#[test]
fn dawn_is_achievable_on_every_difficulty_with_real_fuel_trips() {
    for difficulty in Difficulty::ALL {
        let game = play_to_end(difficulty, false);
        assert_eq!(game.phase, Phase::Dawn, "{game:?}");
        assert!(game.integrity > 0.0);
        assert!(game.searches_left < 3);
    }
}

#[test]
fn early_rescue_is_achievable_on_every_difficulty_with_real_fuel_trips() {
    for difficulty in Difficulty::ALL {
        let game = play_to_end(difficulty, true);
        assert_eq!(game.phase, Phase::Rescued, "{game:?}");
        assert!(game.elapsed < NIGHT_SECONDS);
        assert!(game.searches_left < 3);
    }
}
