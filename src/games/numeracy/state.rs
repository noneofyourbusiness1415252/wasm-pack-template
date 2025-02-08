use super::{Expression, Level};
use wasm_bindgen::JsValue;
use web_sys::Performance;

#[derive(Debug)]
pub struct GameState {
    pub level: Level,
    pub expressions: Vec<Expression>,
    pub selected_indices: Vec<usize>,
    pub score: i32,
    pub round_start: Option<f64>,
    pub level_start: Option<f64>,
    pub completed_rounds: u32,
    performance: Performance,
}

impl GameState {
    pub fn new() -> Self {
        let window = web_sys::window().unwrap();
        let performance = window.performance().unwrap();
        let level = Level::new(1);
        let expressions = level.generate_expressions();
        
        Self {
            level,
            expressions,
            selected_indices: Vec::new(),
            score: 0,
            round_start: None,
            level_start: None,
            completed_rounds: 0,
            performance,
        }
    }

    pub fn start_level(&mut self) {
        self.level_start = Some(self.performance.now());
        self.start_round();
    }

    pub fn start_round(&mut self) {
        self.expressions = self.level.generate_expressions();
        self.selected_indices.clear();
        self.round_start = Some(self.performance.now());
    }

    pub fn toggle_selection(&mut self, index: usize) -> bool {
        if let Some(pos) = self.selected_indices.iter().position(|&i| i == index) {
            self.selected_indices.remove(pos);
            true
        } else if self.selected_indices.len() < 3 {
            self.selected_indices.push(index);
            true
        } else {
            false
        }
    }

    pub fn check_current_round(&self) -> bool {
        if self.selected_indices.len() != 3 {
            return false;
        }

        let selected_expressions: Vec<Expression> = self
            .selected_indices
            .iter()
            .map(|&i| self.expressions[i].clone())
            .collect();

        Level::check_order(&selected_expressions)
    }

    pub fn get_round_time_remaining(&self) -> Option<f64> {
        self.round_start.map(|start| {
            let elapsed = self.performance.now() - start;
            if elapsed >= 15000.0 {
                0.0
            } else {
                15000.0 - elapsed
            }
        })
    }

    pub fn get_level_time_remaining(&self) -> Option<f64> {
        self.level_start.map(|start| {
            let elapsed = self.performance.now() - start;
            if elapsed >= 300000.0 {
                0.0
            } else {
                300000.0 - elapsed
            }
        })
    }

    pub fn update_score(&mut self, round_success: bool) {
        let time_bonus = self.get_round_time_remaining()
            .map(|t| (t / 1000.0) as i32)
            .unwrap_or(0);

        self.score += if round_success {
            10 + time_bonus
        } else {
            -5
        };

        self.completed_rounds += 1;
    }

    pub fn should_adjust_level(&self) -> Option<i32> {
        if self.completed_rounds >= 10 {
            let success_rate = (self.score as f64) / (self.completed_rounds as f64);
            
            Some(if success_rate > 0.8 {
                1
            } else if success_rate < 0.4 {
                -1
            } else {
                0
            })
        } else {
            None
        }
    }
}