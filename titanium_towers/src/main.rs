use engine::{
    clear, draw, generate_deck_slots, handle_winit_event,
    load_cards_from_file, render_character, setup, Color, DraggableSnapType, Drawable, WindowEvent, Event, Rect,
    VirtualKeyCode, move_unit
};
use winit::event_loop::EventLoop;



const WIDTH: usize = 1920;
const HEIGHT: usize = 1080;

const BACKGROUND_COLOR: Color = (91, 99, 112, 255);

fn create_spawn_point(og_spawn: Rect, id: usize) -> Rect {
    let offset = id * 20;
    Rect {x: og_spawn.x, y: (og_spawn.y + offset) % 200 + og_spawn.y , w: og_spawn.w, h: og_spawn.h}
}
struct TowerTime {
    tower: usize,
    time: std::time::Instant
}
fn attack_tower(unit: engine::Unit) -> TowerTime {
    let c = unit.played_card.card;
    let last_attack_time = unit.t;
    let dmg = c.attack;
    let attack_speed = c.attackSpeed;
    if last_attack_time.elapsed() >= std::time::Duration::from_millis(attack_speed) {
        // unit.assign_new_time();
        dbg!(dmg);
        return TowerTime {
            tower: dmg,
            time: std::time::Instant::now()
        }
    }

    return TowerTime {
        tower: 0,
        time: last_attack_time
    }
}

fn main() {
    let mut tower1_hp = 100;
    let mut tower2_hp = 100;

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

    let c2 = (255, 0, 0, 0);
    let c1 = (0, 255, 0, 0);
    let c3 = (0, 0, 255, 0);

    let mut state = setup();
    // state.bg_color = BACKGROUND_COLOR;
    let event_loop = EventLoop::new();

    let mut starting_game_objects: Vec<Drawable> = vec![];

    let text: String = "hello".to_string();

    let mut towers = vec![
        Drawable::Rectangle(r1, c1, Some(DraggableSnapType::Card(false, false))),
        Drawable::Rectangle(r2, c2, Some(DraggableSnapType::Card(false, false))),
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
    //starting_game_objects.append(&mut played_drawable);

    state.drawables = starting_game_objects.clone();
    state.drawables.append(&mut played_drawable);

    // When 1 is pressed
    let u1 = played_card1.play_unit(std::time::Instant::now(), create_spawn_point(spawn1, unit_id));
    unit_id += 1;
    state.p1_units.push(u1);
    // played_card1 = deck.draw_and_remove().play(slots[2].get_rect());
    // TODO: add player cd, and replenish card
    
    // When 8 is pressed
    let u2 = played_card2.play_unit(std::time::Instant::now(), create_spawn_point(spawn2, unit_id));
    unit_id += 1;
    state.p2_units.push(u2);
    // played_card2 = deck.draw_and_remove().play(slots[4].get_rect());
    // TODO: add player cd, and replenish card

    // TODO: add more button presses for each player
        
    
    event_loop.run(move |event, _, control_flow| {
       
        
        if event == Event::MainEventsCleared {
            state.bg_color = BACKGROUND_COLOR;
            
            let mut p1_unit_drawables = vec![];
            let mut p2_unit_drawables = vec![];

            // state.drawables.push(pc5_1);
            for unit in state.p1_units.iter() {
                let c = unit.played_card.card.clone();
                if unit.get_rect_x() <= WIDTH - 300 {
                    p1_unit_drawables.push(unit.move_unit(c.speed * 3));
                } else {
                    // take damage
                    let tower_time = attack_tower(unit.get_unit());

                    tower2_hp -= tower_time.tower;

                    p1_unit_drawables.push(unit.assign_new_time(tower_time.time));
                    dbg!(tower2_hp);
                }
            }
    
            for unit in state.p2_units.iter() {
                let c = unit.played_card.card.clone();

                if unit.get_rect_x() >= 300 {
                    p2_unit_drawables.push(unit.move_unit_back(c.speed * 3));
                    
                } else {
                    // take damage

                    let tower_time = attack_tower(unit.get_unit());

                    tower1_hp -= tower_time.tower;

                    p2_unit_drawables.push(unit.assign_new_time(tower_time.time));
                    dbg!(tower1_hp);
                }
            }
    
            state.drawables = starting_game_objects.clone();
    
            for unit in p1_unit_drawables.iter() {
                state.drawables.push(unit.played_card.get_drawable_rect(c1));
            }

            for unit in p2_unit_drawables.iter() {
                state.drawables.push(unit.played_card.get_drawable_rect(c2));
            }
    
            // state.drawables.push(played_card1.get_drawable());
            // state.drawables.push(played_card2.get_drawable());
            // state.drawables.push(played_card3.get_drawable());
            // state.drawables.push(played_card4.get_drawable());

            state.p1_units = p1_unit_drawables;
            state.p2_units = p2_unit_drawables;

            draw(&mut state);
        }

        handle_winit_event(event, control_flow, &mut state);
    });
}
