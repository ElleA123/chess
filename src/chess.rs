mod bitboard;
mod board;
mod color;
mod magic_tables;
mod mv;
mod piece;
mod square;

pub use board::{Board, START_POS_FEN, make_move, gen_legal_moves};
pub use color::*;
pub use magic_tables::init_magic_tables;
pub use mv::*;
pub use piece::*;
pub use square::*;