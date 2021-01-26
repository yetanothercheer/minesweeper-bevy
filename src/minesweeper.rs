use std::thread;

use rand::{Rng};

#[derive(Clone, Copy)]
pub struct State {
    pub surrounds: usize,
    pub bomb: bool,
    pub reveal: bool,
    pub flag: bool,
}

impl Default for State {
    fn default() -> Self {
        State {
            surrounds: 0,
            bomb: false,
            reveal: false,
            flag: false,
        }
    }
}

pub struct Mines {
    width: usize,
    height: usize,
    pub state: Vec<State>,
}

pub enum Status {
    GameOver,
    Unfinished,
    Win,
}

fn surround_at(x: usize, y: usize) -> Vec<(usize, usize)> {
    vec![
        (x - 1, y - 1),
        (x, y - 1),
        (x + 1, y - 1),
        (x - 1, y),
        (x + 1, y),
        (x - 1, y + 1),
        (x, y + 1),
        (x + 1, y + 1),
    ]
}

impl Mines {
    pub fn new(width: usize, height: usize) -> Mines {
        Mines {
            width,
            height,
            state: vec![Default::default(); width * height],
        }
    }

    pub fn generate(&mut self, bomb_num: usize, exclude: (usize, usize)) {
        let mut rand = rand::thread_rng();
        if bomb_num >= self.width * self.height {
            println!(
                "Warning: bomb num {} exceeds maximum {}",
                bomb_num,
                self.width * self.height
            )
        }
        (0..bomb_num).for_each(|_| loop {
            let x = rand.gen_range(1..=self.width);
            let y = rand.gen_range(1..=self.height);
            if x != exclude.0 && y != exclude.1 && !self.at(x, y).unwrap().bomb {
                self.at(x, y).unwrap().bomb = true;
                surround_at(x, y)
                    .iter()
                    .for_each(|p| match self.at(p.0, p.1) {
                        Some(state) => state.surrounds += 1,
                        None => {}
                    });
                break;
            }
        });
    }

    pub fn status(&self) -> Status {
        let mut unfinished = false;
        for state in &self.state {
            if state.bomb && state.reveal {
                return Status::GameOver;
            }
            if !state.bomb && !state.reveal {
                unfinished = true;
            }
        }
        if unfinished {
            Status::Unfinished
        } else {
            Status::Win
        }
    }

    // Input Range
    // x: 1..=width
    // y: 1..=height
    pub fn at(&mut self, x: usize, y: usize) -> Option<&mut State> {
        if x == 0 || y == 0 || x > self.width || y > self.height {
            return None;
        }
        self.state.get_mut((y - 1) * self.width + (x - 1))
    }

    pub fn reveal(&mut self, x: usize, y: usize) {
        match self.at(x, y) {
            Some(state) => {
                if !state.reveal {
                    state.reveal = true;
                    if !state.bomb && state.surrounds == 0 {
                        surround_at(x, y).iter().for_each(|p| self.reveal(p.0, p.1));
                    }
                }
            }
            None => {}
        }
    }

    fn pretty_print(&mut self) {
        for y in 1..=self.height {
            for x in 1..=self.width {
                let state = self.at(x, y).unwrap();
                print!(
                    "{} ",
                    if state.bomb {
                        String::from("😀")
                    } else {
                        state.surrounds.to_string() + if state.reveal { "." } else { "" }
                    }
                )
            }
            println!("")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut mines = Mines::new(10, 10);
        mines.generate(10, (1, 1));
        mines.reveal(1, 1);
        mines.pretty_print();
    }
}
