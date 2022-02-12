use engine::{setup, updateGameState, Rect, Drawable, newState};


fn main() {

    let r1 = Rect{x: 10, y: 10, w: 100, h: 100};
    let r2 = Rect{x: 300, y: 300, w: 30, h: 30};
    let c1 = (255, 0, 0, 0);
    let c2 = (0, 255, 0, 0);
 
    let mut new_state = newState();

    let mut game_objects: Vec<Drawable> = vec![];
    game_objects.push(Drawable::Rectangle(r1, c1));
    game_objects.push(Drawable::RectOutlined(r2, c2));

    new_state.game_objects = game_objects;
    
    setup();
    std::thread::sleep(std::time::Duration::from_secs(5));
    dbg!{"Finished sleeping"};
    updateGameState(game_objects);
}
