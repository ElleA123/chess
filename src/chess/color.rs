#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Color {
    White,
    Black
}

pub const NUM_COLORS: usize = 2;
pub const COLORS: [Color; 2] = [Color::White, Color::Black];

impl Color {
    #[inline]
    pub const fn is_white(&self) -> bool {
        match self {
            Color::White => true,
            Color::Black => false
        }
    }

    #[inline]
    pub const fn is_black(&self) -> bool {
        match self {
            Color::White => false,
            Color::Black => true
        }
    }

    #[inline]
    pub const fn idx(self) -> usize {
        self as usize
    }

    #[inline(always)]
    pub const fn map<T: Copy>(&self, white: T, black: T) -> T {
        match self {
            Color::White => white,
            Color::Black => black
        }
    }
}

impl std::ops::Not for Color {
    type Output = Color;
    fn not(self) -> Self::Output {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White
        }
    }
}