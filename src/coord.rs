use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Coord(pub usize, pub usize);

pub const BOARD_SIZE: usize = 8;

pub const fn is_on_board(y: usize, x: usize) -> bool {
    y < 8 && x < 8 // type limits cover the bottom half
}

impl PartialEq<(usize, usize)> for Coord {
    fn eq(&self, other: &(usize, usize)) -> bool {
        self.0 == other.0 && self.1 == other.1
    }
}

impl From<Coord> for (usize, usize) {
    fn from(value: Coord) -> Self {
        (value.0, value.1)
    }
}

impl Display for Coord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", (self.1 as u8 + 'a' as u8) as char, BOARD_SIZE - self.0)
    }
}

impl Coord {
    pub const fn from(y: usize, x: usize) -> Option<Self> {
        if is_on_board(y, x) {
            Some(Self(y, x))
        } else {
            None
        }
    }

    pub fn from_san(san: &str) -> Option<Self> {
        let mut chars = san.chars();
        let x = match chars.next() {
            Some(c) => (c as usize) - ('a' as usize),
            None => { return None; }
        };

        let Some(y) = chars.next() else { return None; };
        let y = match y.to_digit(10) {
            Some(i) => BOARD_SIZE - i as usize,
            None => return None
        };

        if y < 8 && x < 8 {
            Some(Self(y, x))
        } else {
            None
        }
    }

    pub fn add_mut(&mut self, step: &(isize, isize)) -> bool {
        if self.0 as isize + step.0 >= 0 && self.1 as isize + step.1 >= 0 {
            let y = (self.0 as isize + step.0) as usize;
            let x = (self.1 as isize + step.1) as usize;
            if is_on_board(y, x) {
                self.0 = y;
                self.1 = x;
                return true;
            }
        }
        false
    }

    pub fn add(&self, step: &(isize, isize)) -> Option<Coord> {
        if self.0 as isize + step.0 >= 0 && self.1 as isize + step.1 >= 0 {
            let y = (self.0 as isize + step.0) as usize;
            let x = (self.1 as isize + step.1) as usize;
            if is_on_board(y, x) {
                return Some(Coord(y, x));
            }
        }
        None
    }

    pub fn all() -> impl Iterator<Item = Self> {
        (0..64).map(|i| Coord(i / 8, i % 8))
    }

    pub fn all_tup() -> impl Iterator<Item = (usize, usize)> {
        (0..64).map(|i| (i / 8, i % 8))
    }

    pub fn file(x: usize) -> impl Iterator<Item = Self> {
        (0..8).map(move |y| Coord(y, x))
    }

    pub fn rank(y: usize) -> impl Iterator<Item = Self> {
        (0..8).map(move |x| Coord(y, x))
    }
}