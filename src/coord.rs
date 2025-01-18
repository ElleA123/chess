#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Coord(usize, usize);

pub const BOARD_SIZE: usize = 8;

pub fn is_on_board(y: usize, x: usize) -> bool {
    y < 8 && x < 8 // type limits cover the bottom half
}

impl Coord {
    pub const fn from(y: usize, x: usize) -> Self {
        Self(y % BOARD_SIZE, x % BOARD_SIZE)
    }

    pub fn all() -> impl Iterator<Item = Self> {
        (0..64).map(|i| Coord(i / 8, i % 8))
    }

    pub fn all_tup() -> impl Iterator<Item = (usize, usize)> {
        (0..64).map(|i| (i / 8, i % 8))
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

    pub fn add_mut(&mut self, step: (isize, isize)) -> bool {
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

    pub fn add(&self, step: (isize, isize)) -> Option<Coord> {
        if self.0 as isize + step.0 >= 0 && self.1 as isize + step.1 >= 0 {
            let y = (self.0 as isize + step.0) as usize;
            let x = (self.1 as isize + step.1) as usize;
            if is_on_board(y, x) {
                return Some(Coord(y, x));
            }
        }
        None
    }

    pub fn to_string(&self) -> String {
        format!("{}{}", (self.1 as u8 + 'a' as u8) as char, BOARD_SIZE - self.0)
    }

    pub fn tup(&self) -> (usize, usize) {
        (self.0, self.1)
    }
}