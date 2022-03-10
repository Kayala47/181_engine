use engine::{
    clear, draw, generate_deck_slots, get_slot_rect, handle_winit_event, load_cards_from_file,
    move_unit, render_character, setup, Color, DraggableSnapType, Drawable, Event, FontFamily,
    Rect, VirtualKeyCode, WindowEvent,
};
use std::time::{Duration, Instant};
use winit::event_loop::EventLoop;

const WIDTH: usize = 1920;
const HEIGHT: usize = 1080;

const CARD_WIDTH: usize = WIDTH / 10;
const CARD_HEIGHT: usize = HEIGHT / 6;
const CARD_PADDING_TOP: usize = 5;
const CARD_PADDING_BOTTOM: usize = 5;
const NUM_SLOTS: usize = 4;

const TOWER_START_HP: usize = 10;

const BACKGROUND_COLOR: Color = (91, 99, 112, 255);

#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
enum GameState {
    Started,
    P1Won,
    P2Won,
}

fn create_spawn_point(og_spawn: Rect, id: usize) -> Rect {
    let offset = id * 20;
    Rect {
        x: og_spawn.x,
        y: (og_spawn.y + offset) % 200 + og_spawn.y,
        w: og_spawn.w,
        h: og_spawn.h,
    }
}
struct TowerTime {
    tower: usize,
    time: Instant,
}
fn attack_tower(unit: engine::Unit) -> TowerTime {
    let c = unit.played_card.card;
    let last_attack_time = unit.t;
    let dmg = c.attack;
    let attack_speed = c.attackSpeed;
    if last_attack_time.elapsed() >= Duration::from_millis(attack_speed) {
        dbg!(c.name.clone());
        dbg!(dmg);
        return TowerTime {
            tower: dmg,
            time: Instant::now(),
        };
    }

    TowerTime {
        tower: 0,
        time: last_attack_time,
    }
}

fn ready_to_play(t: Instant, card_cost: usize) -> bool {
    t.elapsed() >= Duration::from_secs(card_cost as u64)
}

fn generate_health_bar(hp: usize, tower: usize) -> Vec<Drawable> {
    let remaining_health_width = ((hp as f32 / TOWER_START_HP as f32) * 200.0) as usize;
    let tower_x = if tower == 1 { 100 } else { WIDTH - 300 };
    let r1 = Rect {
        x: tower_x,
        y: HEIGHT / 2 - 70,
        w: remaining_health_width,
        h: 10,
    };
    let r2 = Rect {
        x: tower_x + remaining_health_width,
        y: HEIGHT / 2 - 70,
        w: 200 - remaining_health_width,
        h: 10,
    };

    let red = (255, 0, 0, 0);
    let green = (0, 255, 0, 0);

    vec![
        Drawable::Rectangle(r1, green, None),
        Drawable::Rectangle(r2, red, None),
    ]
}

fn main() {
    let mut game_state = GameState::Started;

    let mut tower1_hp = TOWER_START_HP;
    let mut tower2_hp = TOWER_START_HP;

    let r1 = Rect {
        x: 100,
        y: HEIGHT / 2 - 50,
        w: 200,
        h: 200,
    };
    let r2 = Rect {
        x: WIDTH - 300,
        y: HEIGHT / 2 - 50,
        w: 200,
        h: 200,
    };

    let spawn1 = Rect {
        x: 300,
        y: HEIGHT / 2 - 50,
        w: 20,
        h: 20,
    };

    let spawn2 = Rect {
        x: WIDTH - 300,
        y: HEIGHT / 2 - 50,
        w: 20,
        h: 20,
    };

    let mut unit_id = 0;

    let mut deck = load_cards_from_file("../cards2.json");
    let c1 = (0, 0, 255, 0);
    let c2 = (255, 255, 0, 0);

    let mut state = setup();
    let event_loop = EventLoop::new();

    let mut starting_game_objects: Vec<Drawable> = vec![];

    let mut towers = vec![
        Drawable::Rectangle(r1, c1, Some(DraggableSnapType::Card(false, false))),
        Drawable::Rectangle(r2, c2, Some(DraggableSnapType::Card(false, false))),
    ];

    let mut p1_last_played_t = Instant::now();
    let mut p2_last_played_t = Instant::now();

    let mut slots = generate_deck_slots(
        (CARD_WIDTH, CARD_HEIGHT),
        CARD_PADDING_BOTTOM,
        CARD_PADDING_TOP,
        NUM_SLOTS,
        (0, 0, 0, 255),
        (0, 255, 0, 255),
        (255, 255, 255, 255),
        (220, 220, 250, 255),
        false
    );

    deck.shuffle();
    let mut card1 = deck.draw_and_cycle();
    let mut card2 = deck.draw_and_cycle();
    let mut card3 = deck.draw_and_cycle();
    let mut card4 = deck.draw_and_cycle();
    let played_card1 = card1.play(get_slot_rect(
        1,
        (CARD_WIDTH, CARD_HEIGHT),
        NUM_SLOTS,
        false,
        CARD_PADDING_TOP,
        CARD_PADDING_BOTTOM,
    ));
    let played_card2 = card2.play(get_slot_rect(
        2,
        (CARD_WIDTH, CARD_HEIGHT),
        NUM_SLOTS,
        false,
        CARD_PADDING_TOP,
        CARD_PADDING_BOTTOM,
    ));
    let played_card3 = card3.play(get_slot_rect(
        3,
        (CARD_WIDTH, CARD_HEIGHT),
        NUM_SLOTS,
        false,
        CARD_PADDING_TOP,
        CARD_PADDING_BOTTOM,
    ));
    let played_card4 = card4.play(get_slot_rect(
        4,
        (CARD_WIDTH, CARD_HEIGHT),
        NUM_SLOTS,
        false,
        CARD_PADDING_TOP,
        CARD_PADDING_BOTTOM,
    ));

    let mut played_drawable = vec![
        played_card1.get_clash_drawable(),
        played_card2.get_clash_drawable(),
        played_card3.get_clash_drawable(),
        played_card4.get_clash_drawable(),
    ];

    starting_game_objects.append(&mut slots);
    starting_game_objects.append(&mut towers);
    // starting_game_objects.append(&mut played_drawable);

    state.drawables = starting_game_objects.clone();
    state.drawables.append(&mut played_drawable);

    event_loop.run(move |event, _, control_flow| {
        if event == Event::MainEventsCleared {
            if game_state == GameState::P1Won {
                let result_string = "Player 2 has fallen. Player 1 Wins!";
                let result_text = Drawable::Text(
                    Rect {
                        x: 30,
                        y: 30,
                        w: WIDTH - 30,
                        h: 200,
                    },
                    result_string.to_string(),
                    FontFamily::GameTitle,
                    20.0,
                );
                state.drawables.push(result_text);
                draw(&mut state);
                return;
            } else if game_state == GameState::P2Won {
                let result_string = "Player 1 has fallen. Player 2 Wins!";
                let result_text = Drawable::Text(
                    Rect {
                        x: 30,
                        y: 30,
                        w: WIDTH - 30,
                        h: 200,
                    },
                    result_string.to_string(),
                    FontFamily::GameTitle,
                    12.0,
                );
                state.drawables.push(result_text);
                draw(&mut state);
                return;
            }

            if state.now_keys[VirtualKeyCode::Key1 as usize]
                && !state.prev_keys[VirtualKeyCode::Key1 as usize]
                && ready_to_play(p1_last_played_t, card1.playCost)
            {
                let hp = card1.health;
                let played_card1 = card1.play(get_slot_rect(
                    1,
                    (CARD_WIDTH, CARD_HEIGHT),
                    NUM_SLOTS,
                    false,
                    CARD_PADDING_TOP,
                    CARD_PADDING_BOTTOM,
                ));
                let u = played_card1.play_unit(
                    std::time::Instant::now(),
                    hp,
                    create_spawn_point(spawn1, unit_id),
                );
                unit_id += 1;
                state.p1_units.push(u);

                let dur = Duration::from_secs(card1.playCost as u64);
                dbg!(p1_last_played_t);
                p1_last_played_t += dur;
                dbg!(p1_last_played_t);
                card1 = deck.draw_and_cycle();
            }

            if state.now_keys[VirtualKeyCode::Key2 as usize]
                && !state.prev_keys[VirtualKeyCode::Key2 as usize]
                && ready_to_play(p1_last_played_t, card2.playCost)
            {
                let hp = card2.health;
                let played_card2 = card2.play(get_slot_rect(
                    2,
                    (CARD_WIDTH, CARD_HEIGHT),
                    NUM_SLOTS,
                    false,
                    CARD_PADDING_TOP,
                    CARD_PADDING_BOTTOM,
                ));
                let u = played_card2.play_unit(
                    std::time::Instant::now(),
                    hp,
                    create_spawn_point(spawn1, unit_id),
                );
                unit_id += 1;
                state.p1_units.push(u);

                let dur = Duration::from_secs(card2.playCost as u64);
                p1_last_played_t += dur;

                card2 = deck.draw_and_cycle();
            }

            if state.now_keys[VirtualKeyCode::Key3 as usize]
                && !state.prev_keys[VirtualKeyCode::Key3 as usize]
                && ready_to_play(p1_last_played_t, card3.playCost)
            {
                let hp = card3.health;
                let played_card3 = card3.play(get_slot_rect(
                    3,
                    (CARD_WIDTH, CARD_HEIGHT),
                    NUM_SLOTS,
                    false,
                    CARD_PADDING_TOP,
                    CARD_PADDING_BOTTOM,
                ));
                let u = played_card3.play_unit(
                    std::time::Instant::now(),
                    hp,
                    create_spawn_point(spawn1, unit_id),
                );
                unit_id += 1;
                state.p1_units.push(u);

                let dur = Duration::from_secs(card3.playCost as u64);
                p1_last_played_t += dur;

                card3 = deck.draw_and_cycle();
            }

            if state.now_keys[VirtualKeyCode::Key4 as usize]
                && !state.prev_keys[VirtualKeyCode::Key4 as usize]
                && ready_to_play(p1_last_played_t, card4.playCost)
            {
                let hp = card4.health;
                let played_card4 = card4.play(get_slot_rect(
                    4,
                    (CARD_WIDTH, CARD_HEIGHT),
                    NUM_SLOTS,
                    false,
                    CARD_PADDING_TOP,
                    CARD_PADDING_BOTTOM,
                ));
                let u = played_card4.play_unit(
                    std::time::Instant::now(),
                    hp,
                    create_spawn_point(spawn1, unit_id),
                );
                unit_id += 1;
                state.p1_units.push(u);

                let dur = Duration::from_secs(card4.playCost as u64);
                p1_last_played_t += dur;

                card4 = deck.draw_and_cycle();
            }

            if state.now_keys[VirtualKeyCode::Key7 as usize]
                && !state.prev_keys[VirtualKeyCode::Key7 as usize]
                && ready_to_play(p2_last_played_t, card1.playCost)
            {
                let hp = card1.health;
                let played_card1 = card1.play(get_slot_rect(
                    5,
                    (CARD_WIDTH, CARD_HEIGHT),
                    NUM_SLOTS,
                    true,
                    CARD_PADDING_TOP,
                    CARD_PADDING_BOTTOM,
                ));
                let u = played_card1.play_unit(
                    std::time::Instant::now(),
                    hp,
                    create_spawn_point(spawn2, unit_id),
                );
                unit_id += 1;
                state.p2_units.push(u);

                let dur = Duration::from_secs(card1.playCost as u64);
                p2_last_played_t += dur;

                card1 = deck.draw_and_cycle();
            }

            if state.now_keys[VirtualKeyCode::Key8 as usize]
                && !state.prev_keys[VirtualKeyCode::Key8 as usize]
                && ready_to_play(p2_last_played_t, card2.playCost)
            {
                let hp = card2.health;
                let played_card2 = card2.play(get_slot_rect(
                    6,
                    (CARD_WIDTH, CARD_HEIGHT),
                    NUM_SLOTS,
                    true,
                    CARD_PADDING_TOP,
                    CARD_PADDING_BOTTOM,
                ));
                let u = played_card2.play_unit(
                    std::time::Instant::now(),
                    hp,
                    create_spawn_point(spawn2, unit_id),
                );
                unit_id += 1;
                state.p2_units.push(u);

                let dur = Duration::from_secs(card2.playCost as u64);
                p2_last_played_t += dur;

                card2 = deck.draw_and_cycle();
            }

            if state.now_keys[VirtualKeyCode::Key9 as usize]
                && !state.prev_keys[VirtualKeyCode::Key9 as usize]
                && ready_to_play(p2_last_played_t, card3.playCost)
            {
                let hp = card3.health;
                let played_card3 = card3.play(get_slot_rect(
                    7,
                    (CARD_WIDTH, CARD_HEIGHT),
                    NUM_SLOTS,
                    true,
                    CARD_PADDING_TOP,
                    CARD_PADDING_BOTTOM,
                ));
                let u = played_card3.play_unit(
                    std::time::Instant::now(),
                    hp,
                    create_spawn_point(spawn2, unit_id),
                );
                unit_id += 1;
                state.p2_units.push(u);

                let dur = Duration::from_secs(card3.playCost as u64);
                p2_last_played_t += dur;

                card3 = deck.draw_and_cycle();
            }

            if state.now_keys[VirtualKeyCode::Key0 as usize]
                && !state.prev_keys[VirtualKeyCode::Key0 as usize]
                && ready_to_play(p2_last_played_t, card4.playCost)
            {
                let hp = card4.health;
                let played_card4 = card4.play(get_slot_rect(
                    8,
                    (CARD_WIDTH, CARD_HEIGHT),
                    NUM_SLOTS,
                    false,
                    CARD_PADDING_TOP,
                    CARD_PADDING_BOTTOM,
                ));
                let u = played_card4.play_unit(
                    std::time::Instant::now(),
                    hp,
                    create_spawn_point(spawn2, unit_id),
                );
                unit_id += 1;
                state.p2_units.push(u);

                let dur = Duration::from_secs(card4.playCost as u64);
                p2_last_played_t += dur;

                card4 = deck.draw_and_cycle();
            }
            // dbg!(&played_card1);
            let mut cards = vec![
                card1
                    .play(get_slot_rect(
                        1,
                        (CARD_WIDTH, CARD_HEIGHT),
                        NUM_SLOTS,
                        false,
                        CARD_PADDING_TOP,
                        CARD_PADDING_BOTTOM,
                    ))
                    .get_clash_drawable(),
                card2
                    .play(get_slot_rect(
                        2,
                        (CARD_WIDTH, CARD_HEIGHT),
                        NUM_SLOTS,
                        false,
                        CARD_PADDING_TOP,
                        CARD_PADDING_BOTTOM,
                    ))
                    .get_clash_drawable(),
                card3
                    .play(get_slot_rect(
                        3,
                        (CARD_WIDTH, CARD_HEIGHT),
                        NUM_SLOTS,
                        false,
                        CARD_PADDING_TOP,
                        CARD_PADDING_BOTTOM,
                    ))
                    .get_clash_drawable(),
                card4
                    .play(get_slot_rect(
                        4,
                        (CARD_WIDTH, CARD_HEIGHT),
                        NUM_SLOTS,
                        false,
                        CARD_PADDING_TOP,
                        CARD_PADDING_BOTTOM,
                    ))
                    .get_clash_drawable(),
            ];


            let p1_mana = (p1_last_played_t.elapsed().as_secs()).to_string();
            let p2_mana = (p2_last_played_t.elapsed().as_secs()).to_string();

            let mut mana_drawables = vec![
                Drawable::Text(
                    Rect {
                        x: 100,
                        y: HEIGHT / 2 + 170,
                        w: 200,
                        h: 80,
                    },
                    format!("Mana: {}",p1_mana).to_string(),
                    FontFamily::GameTitle,
                    12.0,
                ),
                Drawable::Text(
                    Rect {
                        x: WIDTH - 300,
                        y: HEIGHT / 2 + 170,
                        w: 200,
                        h: 80,
                    },
                    format!("Mana: {}",p2_mana),
                    FontFamily::GameTitle,
                    12.0,
                ),
            ];

            state.bg_color = BACKGROUND_COLOR;
            let mut p1_unit_drawables = vec![];
            let mut p2_unit_drawables = vec![];

            for unit in state.p1_units.iter() {
                let c = unit.played_card.card.clone();
                if unit.get_rect_x() <= WIDTH - 300 {
                    p1_unit_drawables.push(unit.move_unit(c.speed * 3));
                } else {
                    // take damage
                    // check if dead
                    let tower_time = attack_tower(unit.get_unit());
                    if tower_time.tower > tower2_hp {
                        tower2_hp = 0;
                        game_state = GameState::P1Won;
                    } else {
                        tower2_hp -= tower_time.tower;
                    }

                    p1_unit_drawables.push(unit.assign_new_time(tower_time.time));
                    // dbg!(tower2_hp);
                }
            }
            for unit in state.p2_units.iter() {
                let c = unit.played_card.card.clone();

                if unit.get_rect_x() >= 300 {
                    p2_unit_drawables.push(unit.move_unit_back(c.speed * 3));
                } else {
                    // take damage
                    // check if dead

                    let tower_time = attack_tower(unit.get_unit());

                    if tower_time.tower > tower1_hp {
                        tower1_hp = 0;
                        game_state = GameState::P2Won;
                    } else {
                        tower1_hp -= tower_time.tower;
                    }

                    p2_unit_drawables.push(unit.assign_new_time(tower_time.time));
                    // dbg!(tower1_hp);
                }
            }
            state.drawables = starting_game_objects.clone();
            state.drawables.append(&mut cards);
            for unit in p1_unit_drawables.iter() {
                state.drawables.push(unit.played_card.get_drawable_rect(c1));
            }

            for unit in p2_unit_drawables.iter() {
                state.drawables.push(unit.played_card.get_drawable_rect(c2));
            }

            state.drawables.append(&mut mana_drawables);

            let mut health_bar_1 = generate_health_bar(tower1_hp, 1);
            state.drawables.append(&mut health_bar_1);
            let mut health_bar_2 = generate_health_bar(tower2_hp, 2);
            state.drawables.append(&mut health_bar_2);

            state.p1_units = p1_unit_drawables;
            state.p2_units = p2_unit_drawables;

            draw(&mut state);
        }

        handle_winit_event(event, control_flow, &mut state);
    });
}
