use engine::{
    clear, draw, handle_winit_event, load_cards_from_file, setup,
    Color, Drawable, DraggableSnapType, Event, Rect, VirtualKeyCode, check_and_handle_drag, generate_deck_slots
};
use winit::event_loop::EventLoop;

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
    let c1 = (255, 0, 0, 0);
    let c2 = (0, 255, 0, 0);

    let mut state = setup();
    // state.bg_color = BACKGROUND_COLOR;
    let event_loop = EventLoop::new();

    let mut starting_game_objects: Vec<Drawable> = vec![];
    
    let mut boxes = vec![Drawable::Rectangle(r1, c1, Some(DraggableSnapType::Card(true, false))), Drawable::RectOutlined(r2, c2, Some(DraggableSnapType::Card(true, false)))];
    
    let mut slots = generate_deck_slots((30, 40), 5, 5, 5, (255,30,255,255),(0,255,0,255),(255,255,255,255),(220,220,250,255));

    starting_game_objects.append(&mut slots);
    starting_game_objects.append(&mut boxes);

    state.drawables = starting_game_objects.clone();
    event_loop.run(move |event, _, control_flow| {

        if event == Event::MainEventsCleared {
            state.bg_color = BACKGROUND_COLOR;
            // let mut new_objects = game_objects.clone();

            check_and_handle_drag(&mut state);
            draw(&mut state);
            
            
            let deck = load_cards_from_file("../cards2.json");

            let mut game_objects: Vec<Drawable> = vec![];
            game_objects.push(Drawable::Rectangle(r1, c1));
            game_objects.push(Drawable::RectOutlined(r2, c2));

            draw(&mut state, game_objects);

            let deck = load_cards_from_file("../cards2.json");

        }

        handle_winit_event(event, control_flow, &mut state);
    });
}


