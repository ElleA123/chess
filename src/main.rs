mod piece; // mod PIECE...
mod coord; // mod COORD...
mod mv; // mod MOVE...
mod board; // mod BOARD!!!
mod zobrist;
mod engine;
mod uci;

use crate::piece::{Piece, PieceType};
use crate::board::Board;
// use crate::zobrist::ZobristHasher;
use crate::engine::get_best_move;

use std::time::Instant;

fn play_vs_self(depth: u32) {
    let mut board = Board::default();
    loop {
        match get_best_move(&mut board, depth) {
            Some(mv) => {
                println!("{}", mv.get_uci());
                board.make_move(&mv, false);
                println!("{}", board);
            },
            None => {
                println!("ggs");
                return;
            }
        }
        if !board.is_live() {
            println!("ggs");
            return;
        }
    }
}

fn get_input(msg: &str) -> String {
    println!("{}", msg);
    let mut buf = String::new();
    std::io::stdin()
        .read_line(&mut buf)
        .expect("Failed to read line");
    buf.trim().to_owned()
}

fn best_move_of_input() {
    let fen = get_input("Input FEN:");
    let mut board = Board::from_fen(fen.as_str()).unwrap();
    println!("{}", board);

    let Ok(depth) = get_input("Search depth:")
        .parse::<u32>() else { panic!("Error: not a natural number"); };
    if depth == 0 { panic!("Not zero idiot"); }

    let start = Instant::now();

    let best_move = get_best_move(&mut board, depth);

    println!("Time: {:?}", start.elapsed());

    match best_move {
        Some(mv) => println!("{}", mv.get_uci()),
        None => print!("No moves!")
    }
}

fn main() {
    // best_move_of_input();
    // play_vs_self(5);

    uci::setup_uci_engine();
}