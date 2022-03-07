use engine::{
    clear, draw, generate_deck_slots, handle_winit_event,
    load_cards_from_file, render_character, setup, Color, DraggableSnapType, Drawable, WindowEvent, Event, Rect,
    VirtualKeyCode, move_unit
};
use winit::event_loop::EventLoop;



const WIDTH: usize = 1920;
const HEIGHT: usize = 1080;

const BACKGROUND_COLOR: Color = (91, 99, 112, 255);

struct GameState {
    dragged: String,
}

pub struct PlayedCard {
    pub card: Card,
    pub rect: Rect,
}

pub struct Card {
    pub name: String,
    playCost: usize,
    health: usize,
    defense: usize,
    passiveCost: usize,
    specialCost: usize,
    specialTag: String,
    special: String, //should be a function somehow
    attack: usize,
    attackTag: String,
    specialAttribute: String,
}

fn main() {
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

    let r3 = Rect {
        x: 150,
        y: 10,
        w: 30,
        h: 40,
    };

    let mut deck = load_cards_from_file("../cards2.json");

    let c1 = (255, 0, 0, 0);
    let c2 = (0, 255, 0, 0);
    let c3 = (0, 0, 255, 0);

    let mut state = setup();
    // state.bg_color = BACKGROUND_COLOR;
    let event_loop = EventLoop::new();

    let mut starting_game_objects: Vec<Drawable> = vec![];

    let text: String = "hello".to_string();

    let mut towers = vec![
        Drawable::Rectangle(r1, c2, Some(DraggableSnapType::Card(false, false))),
        Drawable::Rectangle(r2, c1, Some(DraggableSnapType::Card(false, false))),
    ];

    let mut slots = generate_deck_slots(
        (WIDTH / 10, HEIGHT / 6),
        5,
        5,
        4,
        (0, 0, 0, 255),
        (0, 255, 0, 255),
        (255, 255, 255, 255),
        (220, 220, 250, 255),
    );

    deck.shuffle();
    let played_card1 = deck.draw_and_remove().play(slots[2].get_rect());
    let played_card2 = deck.draw_and_remove().play(slots[4].get_rect());
    let played_card3 = deck.draw_and_remove().play(slots[6].get_rect());
    let played_card4 = deck.draw_and_remove().play(slots[8].get_rect());
    
    let mut played_drawable = vec![
        played_card1.get_drawable(), 
        played_card2.get_drawable(), 
        played_card3.get_drawable(), 
        played_card4.get_drawable()
    ];

    starting_game_objects.append(&mut slots);
    starting_game_objects.append(&mut towers);
    starting_game_objects.append(&mut played_drawable);

    state.drawables = starting_game_objects.clone();

    played_card1.card.play(r3);

    // When 1 is pressed
    state.p1_units.push(played_card1.card.play(r3));
    
    // When 8 is pressed
    state.p2_units.push(played_card2.card.play(r1));
    
    
    let mut pc5 = deck.draw_and_remove().play(r1);
    
    event_loop.run(move |event, _, control_flow| {
        let mut p1_unit_drawables = vec![];

        // let c = pc5.card;
        // let r = pc5.rect;
        pc5 = pc5.move_this(5);

        p1_unit_drawables.push(
            pc5.get_drawable_rect(c3)
            // Drawable::Rectangle(new_unit_pos, c2, Some(DraggableSnapType::Card(false, false)))
        );

        // let p2_unit_drawables = vec![];
        // let pc5_1 = pc5.play(move_unit(pc5.rect, 0.1 as usize)).get_drawable();
        // state.drawables.push(pc5_1);
        // for unit in state.p1_units.iter() {
        //     let new_unit_pos = move_unit(unit.rect, 2);
        //     p1_unit_drawables.push(Drawable::Rectangle(new_unit_pos, c2, Some(DraggableSnapType::Card(false, false))));
        // }

        for unit in state.p2_units.iter() {
            // unit.
        }

        state.drawables.append(&mut p1_unit_drawables);
        
        if event == Event::MainEventsCleared {
            state.bg_color = BACKGROUND_COLOR;
            
            // let mut new_objects = game_objects.clone();
            // check_and_handle_drag(&mut state);

            draw(&mut state);
        }

        handle_winit_event(event, control_flow, &mut state);
    });
}
