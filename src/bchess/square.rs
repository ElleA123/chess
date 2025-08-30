use crate::bchess::color::Color;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum File { A, B, C, D, E, F, G, H }

pub const NUM_FILES: usize = 8;
pub const FILES: [File; NUM_FILES] = [
    File::A, File::B, File::C, File::D, File::E, File::F, File::G, File::H, 
];

impl File {
    pub const fn from_u8(n: u8) -> Self {
        assert!(n < 8);
        match n {
            0 => File::A,
            1 => File::B,
            2 => File::C,
            3 => File::D,
            4 => File::E,
            5 => File::F,
            6 => File::G,
            7 => File::H,
            _ => unreachable!()
        }
    }

    pub const fn from_ascii(b: u8) -> Self {
        assert!(b >= b'a' && b <= b'h');
        Self::from_u8(b - b'a')
    }

    pub const fn left(self) -> Option<Self> {
        match self {
            File::A => None,
            File::B => Some(File::A),
            File::C => Some(File::B),
            File::D => Some(File::C),
            File::E => Some(File::D),
            File::F => Some(File::E),
            File::G => Some(File::F),
            File::H => Some(File::G)
        }
    }

    pub const fn right(self) -> Option<Self> {
        match self {
            File::A => Some(File::B),
            File::B => Some(File::C),
            File::C => Some(File::D),
            File::D => Some(File::E),
            File::E => Some(File::F),
            File::F => Some(File::G),
            File::G => Some(File::H),
            File::H => None
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Rank { One, Two, Three, Four, Five, Six, Seven, Eight }

pub const NUM_RANKS: usize = 8;
pub const RANKS: [Rank; NUM_RANKS] = [
    Rank::One, Rank::Two, Rank::Three, Rank::Four, Rank::Five, Rank::Six, Rank::Seven, Rank::Eight
];

impl Rank {
    pub const fn from_u8(n: u8) -> Self {
        assert!(n < 8);
        match n {
            0 => Rank::One,
            1 => Rank::Two,
            2 => Rank::Three,
            3 => Rank::Four,
            4 => Rank::Five,
            5 => Rank::Six,
            6 => Rank::Seven,
            7 => Rank::Eight,
            _ => unreachable!()
        }
    }

    pub const fn from_ascii(b: u8) -> Self {
        assert!(b >= b'1' && b <= b'8');
        Self::from_u8(b - b'1')
    }

    pub const fn up(self) -> Option<Self> {
        match self {
            Rank::One => Some(Rank::Two),
            Rank::Two => Some(Rank::Three),
            Rank::Three => Some(Rank::Four),
            Rank::Four => Some(Rank::Five),
            Rank::Five => Some(Rank::Six),
            Rank::Six => Some(Rank::Seven),
            Rank::Seven => Some(Rank::Eight),
            Rank::Eight => None
        }
    }

    pub const fn down(self) -> Option<Self> {
        match self {
            Rank::One => None,
            Rank::Two => Some(Rank::One),
            Rank::Three => Some(Rank::Two),
            Rank::Four => Some(Rank::Three),
            Rank::Five => Some(Rank::Four),
            Rank::Six => Some(Rank::Five),
            Rank::Seven => Some(Rank::Six),
            Rank::Eight => Some(Rank::Seven),
        }
    }

    pub const fn forward(&self, color: Color) -> Option<Self> {
        match color {
            Color::White => self.up(),
            Color::Black => self.down()
        }
    }

    pub const fn backward(&self, color: Color) -> Option<Self> {
        match color {
            Color::White => self.down(),
            Color::Black => self.up()
        }
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Square(u8);

pub const NUM_SQUARES: usize = 64;

impl Square {
    pub const fn from_idx(square: u8) -> Self {
        assert!(square < 64);
        Self(square)
    }

    pub const fn from_coords(file: File, rank: Rank) -> Self {
        assert!((file as u8) < 8 && (rank as u8) < 8);
        Self(8 * rank as u8 + file as u8)
    }

    pub fn from_san(san: &str) -> Option<Self> {
        let bytes = san.as_bytes();
        if !san.is_ascii() || bytes.len() != 2
        || bytes[0] < b'a' || bytes[0] > b'h'
        || bytes[1] < b'1' || bytes[1] > b'8' {
            return None;
        }

        Some(Self::from_coords(File::from_ascii(bytes[0]), Rank::from_ascii(bytes[1])))
    }

    pub const fn rank(&self) -> Rank {
        Rank::from_u8(self.0 >> 3)
    }

    pub const fn file(&self) -> File {
        File::from_u8(self.0 & 7)
    }

    pub const fn idx(&self) -> usize {
        self.0 as usize
    }

    pub const fn up(&self) -> Option<Self> {
        match self.rank().up() {
            Some(rank) => Some(Self::from_coords(self.file(), rank)),
            None => None
        }
    }

    pub const fn down(&self) -> Option<Self> {
        match self.rank().down() {
            Some(rank) => Some(Self::from_coords(self.file(), rank)),
            None => None
        }
    }

    pub const fn left(&self) -> Option<Self> {
        match self.file().left() {
            Some(file) => Some(Self::from_coords(file, self.rank())),
            None => None
        }
    }

    pub const fn right(&self) -> Option<Self> {
        match self.file().right() {
            Some(file) => Some(Self::from_coords(file, self.rank())),
            None => None
        }
    }

    pub const fn forward(&self, color: Color) -> Option<Self> {
        match self.rank().forward(color) {
            Some(rank) => Some(Self::from_coords(self.file(), rank)),
            None => None
        }
    }

    pub const fn backward(&self, color: Color) -> Option<Self> {
        match self.rank().backward(color) {
            Some(rank) => Some(Self::from_coords(self.file(), rank)),
            None => None
        }
    }

    pub const A1: Self = Self::from_coords(File::A, Rank::One);
    pub const B1: Self = Self::from_coords(File::B, Rank::One);
    pub const C1: Self = Self::from_coords(File::C, Rank::One);
    pub const D1: Self = Self::from_coords(File::D, Rank::One);
    pub const E1: Self = Self::from_coords(File::E, Rank::One);
    pub const F1: Self = Self::from_coords(File::F, Rank::One);
    pub const G1: Self = Self::from_coords(File::G, Rank::One);
    pub const H1: Self = Self::from_coords(File::H, Rank::One);
    pub const A8: Self = Self::from_coords(File::A, Rank::Eight);
    pub const B8: Self = Self::from_coords(File::B, Rank::Eight);
    pub const C8: Self = Self::from_coords(File::C, Rank::Eight);
    pub const D8: Self = Self::from_coords(File::D, Rank::Eight);
    pub const E8: Self = Self::from_coords(File::E, Rank::Eight);
    pub const F8: Self = Self::from_coords(File::F, Rank::Eight);
    pub const G8: Self = Self::from_coords(File::G, Rank::Eight);
    pub const H8: Self = Self::from_coords(File::H, Rank::Eight);
}

impl std::fmt::Display for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}",
        (self.file() as u8 + b'a') as char,
        (self.rank() as u8 + b'1') as char)
    }
}