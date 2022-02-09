use engine::{setup, draw}


fn main() {

    let mut now_keys = [false; 255];
    let mut prev_keys = now_keys.clone();

    let c = (255, 0, 0, 0);

    event_loop = setup();

    event_loop.run(move | event, _, control_flow|{

        match event{

            Event::MainEventsCleared => {
                // We can actually handle events now that we know what they all are.
                if now_keys[VirtualKeyCode::Escape as usize] {
                    *control_flow = ControlFlow::Exit;
                }

                if now_keys[VirtualKeyCode::Down as usize] {
                    if !now_keys[VirtualKeyCode::LControl as usize] {
                        ay += accel;
                    }
                    // if now_keys[VirtualKeyCode::LShift as usize] {
                    //     y2 += accel;
                    // }
                }
                if now_keys[VirtualKeyCode::Right as usize] && x < (WIDTH - 1) as f32 {
                    if !now_keys[VirtualKeyCode::LControl as usize] {
                        ax += accel;
                    }
                    // if now_keys[VirtualKeyCode::LShift as usize] {
                    //     x2 += accel;
                    // }
                }

                if !now_keys[VirtualKeyCode::Right as usize]
                    && !now_keys[VirtualKeyCode::Down as usize]
                {
                    ax = 0.0;
                    ay = 0.0;
                }

                if now_keys[VirtualKeyCode::LShift as usize] {
                    //move to muddy ground
                    decay = 0.5;
                    accel = 0.2;
                } else {
                    decay = 0.3;
                    accel = 0.5;
                }
            }

            _ => {engine::handle_events(event)}
        }


    });


}
