use engine::{
    check_and_handle_drag, clear, draw, generate_battle_slots, generate_deck_slots,
    handle_winit_event, load_cards_from_file, render_character, setup, Color, DraggableSnapType,
    Drawable, Event, PlayedCard, Rect, VirtualKeyCode, WindowEvent,
};
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
    let mut turn = 0;

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

    // let played_card = deck.draw_and_remove().play(slots[2].get_rect());
    // let mut played_drawable = vec![played_card.get_drawable()];

    starting_game_objects.append(&mut slots.clone());
    // starting_game_objects.append(&mut boxes.clone());
    starting_game_objects.append(&mut played_drawables.clone());
    starting_game_objects.append(&mut battle_slots.clone());

    state.drawables = starting_game_objects.clone();

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::MainEventsCleared => {
                state.bg_color = BACKGROUND_COLOR;
                // let mut new_objects = game_objects.clone();

                //not necessary? resets movement :/
                // state.drawables = vec![];
                // state.drawables.append(&mut slots.clone());
                // state.drawables.append(&mut boxes.clone());
                // state.drawables.append(&mut battle_slots.clone());

                // played_cards
                //     .iter()
                //     .for_each(|card| state.drawables.push(card.get_drawable()));

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
                match key_state {
                    winit::event::ElementState::Pressed => {
                        // VirtualKeycode is an enum with a defined representation
                        // state.now_keys[virtual_keycode as usize] = true;
                        turn += 1;
                        dbg!(turn);
                        println!("key pressed");
                    }
                    winit::event::ElementState::Released => {
                        // state.now_keys[virtual_keycode as usize] = false;
                    }
                }
            }
            _ => handle_winit_event(event, control_flow, &mut state),
        }
    });
}
