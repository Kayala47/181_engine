use engine::{
    check_and_handle_drag, clear, draw, generate_deck_slots, handle_winit_event,
    load_cards_from_file, render_character, setup, Color, DraggableSnapType, Drawable, Event, Rect,
    VirtualKeyCode,
};
use winit::event_loop::EventLoop;

const WIDTH: usize = 1920;
const HEIGHT: usize = 1080;

const BACKGROUND_COLOR: Color = (91, 99, 112, 255);

struct GameState {
    dragged: String,
}

fn main() {
    let r1 = Rect {
        x: 10,
        y: 10,
        w: 100,
        h: 100,
    };
    let r2 = Rect {
        x: 150,
        y: 150,
        w: 30,
        h: 30,
    };

    let r3 = Rect {
        x: 150,
        y: 10,
        w: 30,
        h: 40,
    };

    let mut deck = load_cards_from_file("../cards2.json");

    let c1 = (255, 0, 0, 0);
    let c2 = (0, 255, 0, 0);

    let mut state = setup();
    // state.bg_color = BACKGROUND_COLOR;
    let event_loop = EventLoop::new();

    let mut starting_game_objects: Vec<Drawable> = vec![];

    let text: String = "hello".to_string();

    let mut boxes = vec![
        Drawable::Rectangle(r1, c1, Some(DraggableSnapType::Card(true, false))),
        Drawable::RectOutlined(r2, c2, Some(DraggableSnapType::Card(true, false))),
    ];

    let mut slots = generate_deck_slots(
        (WIDTH / 10, HEIGHT / 6),
        5,
        5,
        5,
        (0, 0, 0, 255),
        (0, 255, 0, 255),
        (255, 255, 255, 255),
        (220, 220, 250, 255),
    );

    let played_card = deck.draw_and_remove().play(slots[2].get_rect());
    let mut played_drawable = vec![played_card.get_drawable()];

    starting_game_objects.append(&mut slots);
    starting_game_objects.append(&mut boxes);
    starting_game_objects.append(&mut played_drawable);

    state.drawables = starting_game_objects.clone();

    event_loop.run(move |event, _, control_flow| {
        if event == Event::MainEventsCleared {
            state.bg_color = BACKGROUND_COLOR;
            // let mut new_objects = game_objects.clone();

            check_and_handle_drag(&mut state);
            draw(&mut state);
        }
        handle_winit_event(event, control_flow, &mut state);
    });
}