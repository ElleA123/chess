use super::bitboard::Bitboard;
use super::board::Board;
use super::square::*;
use super::color::*;

struct Game {
    board: Board,
    fullmoves: u32,
    position_history: Vec<u64>
}