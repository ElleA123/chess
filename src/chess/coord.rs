#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Coord {
    pub y: usize,
    pub x: usize
}

const fn is_on_board(y: usize, x: usize) -> bool {
    y < 8 && x < 8
}

pub const COORDS: [Coord; 64] = {
    let mut arr = [Coord::new(0, 0); 64];
    let mut i = 0;
    while i < 64 {
        arr[i].y = i / 8;
        arr[i].x = i % 8;
        i += 1;
    }
    arr
};

impl Coord {
    pub const fn new(y: usize, x: usize) -> Self {
        assert!(is_on_board(y, x));
        Self { y, x }
    }

    pub const fn from_san(san: &str) -> Option<Self> {
        let bytes = san.as_bytes();
        if !san.is_ascii() || bytes.len() != 2 || bytes[0] < 'a' as u8 || bytes[1] < '1' as u8 { return None; }

        let x = bytes[0] - 'a' as u8;
        let y = 7 - (bytes[1] - '1' as u8);

        if y < 8 && x < 8 {
            Some(Self::new(y as usize, x as usize))
        } else {
            None
        }
    }

    pub const fn idx(&self) -> usize {
        self.y * 8 + self.x
    }

    pub const fn add(&mut self, step: (isize, isize)) -> bool {
        if self.y as isize + step.0 >= 0 && self.x as isize + step.1 >= 0 {
            let y = (self.y as isize + step.0) as usize;
            let x = (self.x as isize + step.1) as usize;
            if is_on_board(y, x) {
                self.y = y;
                self.x = x;
                return true;
            }
        }
        return false;
    }

    // pub const fn add(&self, step: &(isize, isize)) -> Option<Coord> {
    //     if self.y as isize + step.0 >= 0 && self.x as isize + step.1 >= 0 {
    //         let y = (self.y as isize + step.0) as usize;
    //         let x = (self.x as isize + step.1) as usize;
    //         if is_on_board(y, x) {
    //             return Some(Coord::new(y, x));
    //         }
    //     }
    //     return None;
    // }

    // pub const fn file(x: usize) -> [Self; 8] {
    //     [Coord::new(0, x), Coord::new(1, x), Coord::new(2, x), Coord::new(3, x),
    //     Coord::new(4, x), Coord::new(5, x), Coord::new(6, x), Coord::new(7, x)]
    // }

    // pub const fn rank(y: usize) -> [Self; 8] {
    //     [Coord::new(y, 0), Coord::new(y, 1), Coord::new(y, 2), Coord::new(y, 3),
    //     Coord::new(y, 4), Coord::new(y, 5), Coord::new(y, 6), Coord::new(y, 7)]
    // }
}

impl std::fmt::Display for Coord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", (self.x as u8 + 'a' as u8) as char, 8 - self.y)
    }
}