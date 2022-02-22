use engine::{
    clear, draw, load_cards_from_file, setup, synchronize_prev_frame_end, Color, Drawable, Event,
    Rect, VirtualKeyCode,
};
use winit::event_loop::EventLoop;

const BACKGROUND_COLOR: Color = (91, 99, 112, 255);

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

    event_loop.run(move |event, _, control_flow| {
        // event_loop_run(event);

        match event {
            Event::MainEventsCleared => {
                // synchronize_prev_frame_end(state);
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

                let mut game_objects: Vec<Drawable> = vec![];
                game_objects.push(Drawable::Rectangle(r1, c1));
                game_objects.push(Drawable::RectOutlined(r2, c2));

                draw(&mut state, game_objects);

                load_cards_from_file("../cards.json")
            }
            _ => (),
        }
    });
}
