use engine::{
    check_and_handle_drag, clear, draw, draw_layout_text, generate_battle_slots,
    generate_deck_slots, handle_mana, handle_winit_event, load_cards_from_file, render_character,
    setup, Color, DraggableSnapType, Drawable, Event, PlayedCard, Rect, VirtualKeyCode, FontFamily,
    WindowEvent,
};
use std::cmp::max;
use winit::event_loop::EventLoop;

const WIDTH: usize = 1920;
const HEIGHT: usize = 1080;

const BACKGROUND_COLOR: Color = (91, 99, 112, 255);

const CARD_SIZE: (usize, usize) = (WIDTH / 9, HEIGHT / 6);
const CARD_PADDING_BOTTOM: usize = 15;
const CARD_PADDING_TOP: usize = 15;

struct GameState {
    dragged: String,
}

fn main() {
    let mut turn: usize = 0;

    let mut p1_m_idx: usize = 0;
    let mut p2_m_idx: usize = 0;

    let mut p1_mana: usize = 5;
    let mut p2_mana: usize = 5;

    let mut deck1 = load_cards_from_file("../cards2.json");
    deck1.shuffle();

    let mut deck2 = load_cards_from_file("../cards2.json");
    deck2.shuffle();

    let c1 = (255, 0, 0, 0);
    let c2 = (0, 255, 0, 0);

    let mut state = setup();
    // state.bg_color = BACKGROUND_COLOR;
    let event_loop = EventLoop::new();

    let mut starting_game_objects: Vec<Drawable> = vec![];

    // let mut boxes = vec![
    //     Drawable::Rectangle(r1, c1, Some(DraggableSnapType::Card(true, false))),
    //     Drawable::RectOutlined(r2, c2, Some(DraggableSnapType::Card(true, false))),
    // ];

    let mut slots = generate_deck_slots(
        CARD_SIZE,
        CARD_PADDING_BOTTOM,
        CARD_PADDING_TOP,
        5,
        (0, 0, 0, 255),
        (0, 255, 0, 255),
        (255, 255, 255, 255),
        (220, 220, 250, 255),
        true
    );

    let mut battle_slots = generate_battle_slots(
        CARD_SIZE,
        CARD_PADDING_BOTTOM,
        CARD_PADDING_TOP,
        5,
        (255, 0, 0, 0),
    );

    dbg!(slots.len());

    //these are the rectangles to fit cards in
    let mut p1_deck_slots = slots[2..22].iter().step_by(4);
    let mut p2_deck_slots = slots[4..24].iter().step_by(4);

    let p1_deck = &slots[23];
    let p2_deck = &slots[25];

    let p1_d_r = p1_deck.get_rect();
    let p2_d_r = p2_deck.get_rect();

    let p1_mana_r = Rect {
        x: p1_d_r.x,
        y: p1_d_r.y + CARD_PADDING_BOTTOM + CARD_PADDING_TOP + CARD_SIZE.1,
        w: p1_d_r.w,
        h: p1_d_r.h,
    };

    let p2_mana_r = Rect {
        x: p2_d_r.x,
        y: p2_d_r.y - CARD_PADDING_BOTTOM - CARD_PADDING_TOP - CARD_SIZE.1,
        w: p2_d_r.w,
        h: p2_d_r.h,
    };

    let mut played_drawables = vec![];
    let mut played_cards: Vec<PlayedCard> = vec![];

    for _ in 0..5 {
        let slot = p1_deck_slots.next().unwrap();

        let p1card = deck1.draw_and_remove().play(slot.get_rect());

        played_drawables.push(p1card.get_drawable());
        played_cards.push(p1card);
        // dbg!(played_drawables[played_drawables.len() - 1].get_coords());
        // dbg!(slot.get_coords());

        let p2card = deck2
            .draw_and_remove()
            .play(p2_deck_slots.next().unwrap().get_rect());

        played_drawables.push(p2card.get_drawable());
        played_cards.push(p2card);
    }

    starting_game_objects.append(&mut slots.clone());
    // starting_game_objects.append(&mut boxes.clone());
    starting_game_objects.append(&mut played_drawables.clone());
    starting_game_objects.append(&mut battle_slots.clone());

    state.drawables = starting_game_objects.clone();

    dbg!(state.drawables.len());

    p1_m_idx = state.drawables.len();
    state
        .drawables
        .push(Drawable::Text(p1_mana_r, p1_mana.to_string(), FontFamily::GameTitle, 10.0));

    dbg!(state.drawables.len());
    p2_m_idx = state.drawables.len();
    state
        .drawables
        .push(Drawable::Text(p2_mana_r, p2_mana.to_string(), FontFamily::GameTitle, 10.0));

    dbg!(state.drawables.len());

    //loop starts here!

    event_loop.run(move |event, _, control_flow| {
        state.drawables.remove(p2_m_idx);
        state.drawables.remove(p1_m_idx - 1);

        p1_m_idx = state.drawables.len();
        state.drawables.push(Drawable::Text(
            p1_mana_r,
            format!("Player 1 has {} mana", p1_mana),
            FontFamily::GameTitle,
            40.0
        ));

        p2_m_idx = state.drawables.len();
        state.drawables.push(Drawable::Text(
            p2_mana_r,
            format!("Player 2 has {} mana", p2_mana),
            FontFamily::GameTitle,
            40.0
        ));

        match event {
            Event::MainEventsCleared => {
                state.bg_color = BACKGROUND_COLOR;

                check_and_handle_drag(&mut state);
                draw(&mut state);
            }
            Event::WindowEvent {
                // Note this deeply nested pattern match
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            winit::event::KeyboardInput {
                                // Which serves to filter out only events we actually want
                                virtual_keycode: Some(VirtualKeyCode::Space),
                                state: key_state,
                                ..
                            },
                        ..
                    },
                ..
            } => {
                // It also binds these handy variable names!
                if key_state == winit::event::ElementState::Pressed {
                    // VirtualKeycode is an enum with a defined representation
                    // state.now_keys[virtual_keycode as usize] = true;
                    turn += 1;
                    if turn % 2 == 0 {
                        //next turn, gain more mana
                        p1_mana += 5;
                        p2_mana += 5;
                    }
                }
            }
            Event::WindowEvent {
                // Note this deeply nested pattern match
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            winit::event::KeyboardInput {
                                // Which serves to filter out only events we actually want
                                virtual_keycode: Some(VirtualKeyCode::Down),
                                state: key_state,
                                ..
                            },
                        ..
                    },
                ..
            } => {
                // It also binds these handy variable names!
                if key_state == winit::event::ElementState::Pressed {
                    if turn % 2 == 0 {
                        if p1_mana > 0 {
                            p1_mana -= 1;
                        } else {
                            p1_mana = 0;
                        }
                    } else {
                        if p2_mana > 0 {
                            p2_mana -= 1;
                        } else {
                            p2_mana = 0;
                        }
                    }

                    // state.drawables
                }
            }
            _ => handle_winit_event(event, control_flow, &mut state),
        }
    });
}