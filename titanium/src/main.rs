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

    let mut starting_game_objects: Vec<Drawable> =
    vec![Drawable::Rectangle(r1, c1, Some(DraggableSnapType::Card(true, false))), Drawable::RectOutlined(r2, c2, Some(DraggableSnapType::Card(true, false)))];
    
    let mut slots = generate_deck_slots((25, 40), 5, 5, 5, (255,0,255,255),(0,0,255,255),(255,255,255,255),(220,220,250,255));

    starting_game_objects.append(&mut slots);

    state.drawables = starting_game_objects.clone();
    event_loop.run(move |event, _, control_flow| {
        if event == Event::MainEventsCleared {
            state.bg_color = BACKGROUND_COLOR;
            // let mut new_objects = game_objects.clone();

            check_and_handle_drag(&mut state);
            draw(&mut state);
            

            
            // if state.left_mouse_down {
            //     println!{"mouse is down"}
            // }

            // if !state.left_mouse_down && state.prev_left_mouse_down {
            //     println!{"mouse is released"}
            // }

            // // We can actually handle events now that we know what they all are.
            // let shift_enabled = now_keys[VirtualKeyCode::LShift as usize]
            //     || now_keys[VirtualKeyCode::RShift as usize];

            // if now_keys[VirtualKeyCode::Escape as usize] {
            //     *control_flow = ControlFlow::Exit;
            // }
            // if now_keys[VirtualKeyCode::Down as usize] {
            //     if shift_enabled {
            //         y = if y + w < HEIGHT - 1 {
            //             y + movement_speed
            //         } else {
            //             y
            //         };
            //     } else {
            //         color = if color == 0 {
            //             colors.len() - 1
            //         } else {
            //             color - 1
            //         };
            //     }
            // }
            // if now_keys[VirtualKeyCode::Up as usize] {
            //     if shift_enabled {
            //         y = if y > 0 { y - movement_speed } else { y };
            //     } else {
            //         color = if color == 0 {
            //             colors.len() - 1
            //         } else {
            //             color - 1
            //         };
            //     }
            // }
            // if now_keys[VirtualKeyCode::Left as usize] && w > 0 {
            //     if shift_enabled {
            //         x = if x > 0 { x - 1 } else { x };
            //     } else {
            //         w -= movement_speed;
            //     }
            // }
            // if now_keys[VirtualKeyCode::Right as usize] && w < WIDTH - 1 {
            //     if shift_enabled {
            //         x = if x + w < WIDTH - 1 { x + 1 } else { x };
            //     } else {
            //         w += movement_speed;
            //     }
            // }
            // It's debatable whether the following code should live here or in the drawing section.
            // First clear the framebuffer...

            

            
            let deck = load_cards_from_file("../cards2.json");
        }

        handle_winit_event(event, control_flow, &mut state);
    });
}


