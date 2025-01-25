use js_sys::Math;
use std::{cell::RefCell, collections::HashSet, rc::Rc};
use wasm_bindgen::prelude::*;
use web_sys::{console, Document, Element, HtmlElement};
#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}
#[wasm_bindgen]
#[derive(Clone)]
pub struct MazeGame {
    // Game state
    size: usize,
    level: usize,
    mazes_completed: usize,

    // Maze elements
    walls: Vec<bool>,
    current_position: (usize, usize),
    key_position: (usize, usize),
    door_position: (usize, usize),
    visited: HashSet<(usize, usize)>,
    has_key: bool,

    // Timer state
    time_remaining: i32,
    last_tick: f64,

    // DOM reference
    document: Document,
}
#[wasm_bindgen]
impl MazeGame {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<MazeGame, JsValue> {
        console::log_1(&"Creating new game".into());
        let window = web_sys::window().expect("no global window exists");
        let document = window.document().expect("no document exists");
        let size = 2;
        let mut game = MazeGame::create_maze(size, document);
        game.render()?;
        game.start()?;
        Ok(game)
    }

    fn create_maze(size: usize, document: Document) -> MazeGame {
        let mut walls = vec![false; size * size * 4]; // Start with no walls

        // Add random walls
        for i in 0..walls.len() {
            walls[i] = Math::random() < 0.5;
        }

        // Generate door positions
        let door_x = (Math::random() * (size as f64)).floor() as usize;
        let door_y = (Math::random() * (size as f64)).floor() as usize;

        let mut attempts = 0;
        // Generate key position that's accessible without going through door
        let key_position = loop {
            let key_x = (Math::random() * (size as f64)).floor() as usize;
            let key_y = (Math::random() * (size as f64)).floor() as usize;
            let pos = (key_x, key_y);

            // Ensure key is not at start
            if pos == (0, 0) {
                continue;
            }

            // Create a temporary set of walls for path checking
            let test_walls = walls.clone();
            let mut visited = HashSet::new();
            visited.insert((0, 0));

            // Flood fill from start position
            let mut stack = vec![(0, 0)];
            while let Some(current) = stack.pop() {
                for (dx, dy) in &[(0, 1), (0, -1), (1, 0), (-1, 0)] {
                    let next_x = current.0 as i32 + dx;
                    let next_y = current.1 as i32 + dy;

                    if next_x >= 0 && next_x < size as i32 && next_y >= 0 && next_y < size as i32 {
                        let next = (next_x as usize, next_y as usize);

                        // Skip if it's the door position
                        if next == (door_x, door_y) {
                            continue;
                        }

                        // Check if path is not blocked by wall
                        let wall_idx = (current.1 * size + current.0) * 4
                            + if *dx > 0 {
                                1
                            } else if *dx < 0 {
                                3
                            } else if *dy > 0 {
                                2
                            } else {
                                0
                            };

                        if !test_walls[wall_idx] && !visited.contains(&next) {
                            visited.insert(next);
                            stack.push(next);
                        }
                    }
                }
            }

            // If we can reach the key position without going through door
            if visited.contains(&pos) {
                break pos;
            }

            attempts += 1;
            if attempts > 10 {
                // Fallback to a safe position if random generation fails
                break (1, 0);
            }
        };

        // Generate door position after key is placed
        let door_position = loop {
            let door_x = (Math::random() * (size as f64)).floor() as usize;
            let door_y = (Math::random() * (size as f64)).floor() as usize;
            let pos = (door_x, door_y);

            // Ensure door is not at start and not at key position
            if pos != (0, 0) && pos != key_position {
                break pos;
            }
        };

        let mut game = MazeGame {
            size,
            walls,
            current_position: (0, 0),
            key_position,
            door_position,
            visited: HashSet::new(),
            has_key: false,
            level: 1,
            mazes_completed: 0,
            document,
            time_remaining: 300,
            last_tick: js_sys::Date::now() / 1000.0,
        };

        // Clear path function - ensures a 2-cell wide path
        fn clear_path(
            walls: &mut Vec<bool>,
            from: (usize, usize),
            to: (usize, usize),
            size: usize,
        ) {
            let mut current = from;
            while current != to {
                let dx = (to.0 as i32 - current.0 as i32).signum();
                let dy = (to.1 as i32 - current.1 as i32).signum();

                // Clear both current cell's wall and neighbor's wall
                if dx != 0 {
                    let wall_idx = (current.1 * size + current.0) * 4 + if dx > 0 { 1 } else { 3 };
                    walls[wall_idx] = false;
                    // Clear adjacent cell's opposite wall if not at edge
                    if (dx > 0 && current.0 + 1 < size) || (dx < 0 && current.0 > 0) {
                        let next_x = (current.0 as i32 + dx) as usize;
                        let adj_wall_idx =
                            (current.1 * size + next_x) * 4 + if dx > 0 { 3 } else { 1 };
                        walls[adj_wall_idx] = false;

                        // Always clear an escape route (up or down)
                        let escape_dir = if current.1 > 0 { 0 } else { 2 }; // up if not at top, down otherwise
                        walls[(current.1 * size + current.0) * 4 + escape_dir] = false;
                        if escape_dir == 0 && current.1 > 0 {
                            // Clear the corresponding wall in the cell above
                            walls[((current.1 - 1) * size + current.0) * 4 + 2] = false;
                        } else if escape_dir == 2 && current.1 + 1 < size {
                            // Clear the corresponding wall in the cell below
                            walls[((current.1 + 1) * size + current.0) * 4 + 0] = false;
                        }
                    }
                    current.0 = (current.0 as i32 + dx) as usize;
                } else if dy != 0 {
                    let wall_idx = (current.1 * size + current.0) * 4 + if dy > 0 { 2 } else { 0 };
                    walls[wall_idx] = false;
                    // Clear adjacent cell's opposite wall if not at edge
                    if (dy > 0 && current.1 + 1 < size) || (dy < 0 && current.1 > 0) {
                        let next_y = (current.1 as i32 + dy) as usize;
                        let adj_wall_idx =
                            (next_y * size + current.0) * 4 + if dy > 0 { 0 } else { 2 };
                        walls[adj_wall_idx] = false;
                    }
                    current.1 = (current.1 as i32 + dy) as usize;
                }
                // Ensure escape route from the destination
                let escape_dirs = [(0, -1), (0, 1), (-1, 0), (1, 0)]; // up, down, left, right
                for (dx, dy) in escape_dirs.iter() {
                    let next_x = to.0 as i32 + dx;
                    let next_y = to.1 as i32 + dy;
                    if next_x >= 0 && next_x < size as i32 && next_y >= 0 && next_y < size as i32 {
                        let wall_idx = (to.1 * size + to.0) * 4
                            + if *dy < 0 {
                                0
                            } else if *dx > 0 {
                                1
                            } else if *dy > 0 {
                                2
                            } else {
                                3
                            };
                        walls[wall_idx] = false;
                    }
                }
            }
        }

        // Clear paths with extra space around them
        clear_path(&mut game.walls, (0, 0), key_position, size);
        clear_path(&mut game.walls, key_position, door_position, size);

        game.visited.insert((0, 0));
        game
    }
    pub fn render(&self) -> Result<(), JsValue> {
        let maze = self.document.get_element_by_id("maze").unwrap();
        maze.set_attribute(
            "style",
            &format!("grid-template-columns: repeat({}, 60px)", self.size),
        )?;
        maze.set_inner_html("");

        for y in 0..self.size {
            for x in 0..self.size {
                let cell = self.create_cell(x, y)?;
                maze.append_child(&cell)?;
            }
        }

        // Update stats
        if let Some(level_el) = self.document.get_element_by_id("level") {
            level_el.set_inner_html(&self.level.to_string());
        }
        if let Some(completed_el) = self.document.get_element_by_id("completed") {
            completed_el.set_inner_html(&self.mazes_completed.to_string());
        }
        if let Some(timer_el) = self.document.get_element_by_id("timer") {
            let minutes = self.time_remaining / 60;
            let seconds = self.time_remaining % 60;
            timer_el.set_inner_html(&format!("{}:{:02} !", minutes, seconds)); // Removed v3 suffix
        }
        Ok(())
    }
    #[wasm_bindgen]
    pub fn start(&mut self) -> Result<(), JsValue> {
        let game_state = Rc::new(RefCell::new(self.clone()));

        // Add cell click handler
        let click_handler = {
            let game_state = game_state.clone();
            Closure::wrap(Box::new(move |event: web_sys::CustomEvent| {
                if let Ok(mut game) = game_state.try_borrow_mut() {
                    let coords = event.detail().as_string().unwrap();
                    let mut coords = coords.split(',');
                    let x = coords.next().unwrap().parse::<usize>().unwrap();
                    let y = coords.next().unwrap().parse::<usize>().unwrap();

                    let result = game.try_move(x, y);
                    game.render().unwrap();

                    if let Some(window) = web_sys::window() {
                        match result {
                            -1 => window
                                .alert_with_message("Hit a wall! Starting over.")
                                .unwrap(),
                            2 => window.alert_with_message("Level complete!").unwrap(),
                            _ => {}
                        }
                    }
                }
            }) as Box<dyn FnMut(_)>)
        };
        if let Some(maze_el) = self.document.get_element_by_id("maze") {
            maze_el.add_event_listener_with_callback(
                "cell-click",
                click_handler.as_ref().unchecked_ref(),
            )?;
            click_handler.forget();
        }
        let f = {
            let game_state = game_state.clone();
            Closure::wrap(Box::new(move || {
                if let Ok(mut game) = game_state.try_borrow_mut() {
                    let now = js_sys::Date::now() / 1000.0;
                    let delta = (now - game.last_tick) as i32;
                    if delta >= 1 {
                        game.time_remaining -= 1;
                        game.last_tick = now;

                        if game.time_remaining <= 0 {
                            // Reset maze and timer
                            let new_game = MazeGame::create_maze(game.size, game.document.clone());
                            game.walls = new_game.walls;
                            game.key_position = new_game.key_position;
                            game.door_position = new_game.door_position;
                            game.reset_position();
                            game.time_remaining = 300;
                            game.last_tick = now;

                            game.render().unwrap();
                            web_sys::window()
                                .unwrap()
                                .alert_with_message("Time's up! Moving to next maze...")
                                .unwrap();
                        }

                        // Update timer display
                        if let Some(timer_el) = game.document.get_element_by_id("timer") {
                            let minutes = game.time_remaining / 60;
                            let seconds = game.time_remaining % 60;
                            timer_el.set_inner_html(&format!("{}:{:02}", minutes, seconds));
                        }
                    }
                }
            }) as Box<dyn FnMut()>)
        };

        // Set up interval timer
        let window = web_sys::window().unwrap();
        console::log_1(&"Setting up interval...".into());
        let result = window.set_interval_with_callback_and_timeout_and_arguments_0(
            f.as_ref().unchecked_ref(),
            1000,
        );

        match result {
            Ok(_) => console::log_1(&"Interval set up successfully".into()),
            Err(e) => console::log_2(&"Failed to set up interval:".into(), &e),
        }

        f.forget();
        console::log_1(&"Setup complete".into());
        Ok(())
    }
    fn create_cell(&self, x: usize, y: usize) -> Result<Element, JsValue> {
        let cell = self.document.create_element("div")?;
        cell.set_class_name("cell");

        // Add visited and current classes
        if self.visited.contains(&(x, y)) {
            cell.class_list().add_1("visited")?;
        }
        if (x, y) == self.current_position {
            cell.class_list().add_1("current")?;
        }

        // Add key and door symbols - only one instance of the key should exist
        if (x, y) == self.key_position && !self.has_key {
            cell.set_inner_html("🔑");
        } else if (x, y) == self.current_position && self.has_key {
            cell.set_inner_html("🔑");
        } else if (x, y) == self.door_position {
            cell.set_inner_html("🚪");
        }

        // Add chevrons for adjacent cells
        if self.is_adjacent(x, y) {
            let chevron = self.document.create_element("span")?;
            chevron.set_class_name("chevron");
            let direction = match (
                x as i32 - self.current_position.0 as i32,
                y as i32 - self.current_position.1 as i32,
            ) {
                (1, 0) => "right",
                (-1, 0) => "left",
                (0, 1) => "down",
                (0, -1) => "up",
                _ => unreachable!(),
            };
            chevron.class_list().add_1(direction)?;
            chevron.set_text_content(Some("›"));
            cell.append_child(&chevron)?;
        }

        let x = x.clone();
        let y = y.clone();
        let click_callback = Closure::wrap(Box::new(move |_event: web_sys::MouseEvent| {
            if let Some(window) = web_sys::window() {
                if let Some(doc) = window.document() {
                    if let Some(maze_el) = doc.get_element_by_id("maze") {
                        let event_init = web_sys::CustomEventInit::new();
                        event_init.set_detail(&JsValue::from_str(&format!("{},{}", x, y)));
                        let event = web_sys::CustomEvent::new_with_event_init_dict(
                            "cell-click",
                            &event_init,
                        )
                        .unwrap();
                        maze_el.dispatch_event(&event).unwrap();
                    }
                }
            }
        }) as Box<dyn FnMut(_)>);
        let cell_element: &HtmlElement = cell.dyn_ref().unwrap();
        cell_element.set_onclick(Some(click_callback.as_ref().unchecked_ref()));
        click_callback.forget();
        Ok(cell)
    }
    fn is_adjacent(&self, x: usize, y: usize) -> bool {
        let current_x = self.current_position.0;
        let current_y = self.current_position.1;

        // Check if target position is adjacent (up, down, left, right)
        let dx = if x >= current_x {
            x - current_x
        } else {
            current_x - x
        };
        let dy = if y >= current_y {
            y - current_y
        } else {
            current_y - y
        };

        // Only one coordinate can change by 1, the other must be 0
        (dx == 1 && dy == 0) || (dx == 0 && dy == 1)
    }

    fn get_wall_index(&self, from_x: usize, from_y: usize, to_x: usize, to_y: usize) -> usize {
        let cell_walls = 4; // each cell has 4 possible walls
        let base_index = (from_y * self.size + from_x) * cell_walls;

        if to_x > from_x {
            base_index + 1 // right wall
        } else if to_x < from_x {
            base_index + 3 // left wall
        } else if to_y > from_y {
            base_index + 2 // bottom wall
        } else {
            base_index + 0 // top wall
        }
    }

    fn level_up(&mut self) {
        self.size += 1;
        self.level += 1;
        self.mazes_completed = 0;

        // Create new maze with increased size
        let new_game = MazeGame::create_maze(self.size, self.document.clone());
        self.walls = new_game.walls;
        self.current_position = (0, 0);
        self.key_position = new_game.key_position;
        self.door_position = new_game.door_position;
        self.visited.clear();
        self.visited.insert((0, 0));
        self.has_key = false;
        self.render().expect("Failed to render new level");
    }

    #[wasm_bindgen]
    pub fn try_move(&mut self, x: usize, y: usize) -> i32 {
        if !self.is_adjacent(x, y) {
            return 0; // Invalid move
        }

        // Check for wall
        let wall_idx = self.get_wall_index(self.current_position.0, self.current_position.1, x, y);
        if self.walls[wall_idx] {
            self.reset_position();
            return -1; // Hit wall
        }

        // Update position
        self.current_position = (x, y);
        self.visited.insert((x, y));

        // Check for key
        if (x, y) == self.key_position {
            self.has_key = true;
        }

        // Check win condition
        if (x, y) == self.door_position && self.has_key {
            self.mazes_completed += 1;

            if self.mazes_completed >= (self.size - 1).pow(2) {
                self.level_up();
            } else {
                // New maze - reset timer and maze
                self.reset();
            }
            return 2; // Won
        }
        1 // Valid move
    }
    #[wasm_bindgen]
    pub fn reset(&mut self) {
        let new_game = MazeGame::create_maze(self.size, self.document.clone());
        self.walls = new_game.walls;
        self.key_position = new_game.key_position;
        self.door_position = new_game.door_position;
        self.reset_position();

        // Reset timer state completely
        self.time_remaining = 300;
        self.last_tick = js_sys::Date::now() / 1000.0;

        // Force timer display update
        if let Some(timer_el) = self.document.get_element_by_id("timer") {
            timer_el.set_inner_html("5:00");
        }

        // Update display
        self.render().expect("Failed to render reset");
    }
    fn reset_position(&mut self) {
        self.current_position = (0, 0);
        self.visited.clear();
        self.visited.insert((0, 0));
        self.has_key = false;
        self.render().expect("Failed to render position reset");
    }
    // Additional helper methods...
}
