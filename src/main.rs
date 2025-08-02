mod chess;
mod zobrist;
mod engine;
mod uci;

use chess::{Board, Move};
// use crate::zobrist::ZobristHasher;
use crate::{chess::Coord, engine::get_best_move};

use std::time::Instant;

fn play_vs_self(depth: usize) {
    let mut board = Board::default();
    while board.is_live() {
        match get_best_move(&mut board, depth) {
            Some(mv) => {
                println!("{}", mv.uci());
                board.make_move(&mv, false);
                println!("{}", board);
            },
            None => {
                println!("this should not happen -- no moves but game is live?");
                println!("{}", board.get_fen());
                return;
            }
        }
    }
    println!("ggs - game no longer live");
    println!("{}", board.get_fen());
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
    let Some(mut board) = Board::from_fen(fen.as_str()) else { panic!("invalid FEN"); };
    println!("{}", board);

    let Ok(depth) = get_input("Search depth:")
        .parse::<usize>() else { panic!("depth is not a natural number"); };
    if depth == 0 { panic!("Not zero idiot"); }

    let start = Instant::now();

    let best_move = get_best_move(&mut board, depth);

    println!("Time: {:?}", start.elapsed());

    match best_move {
        Some(mv) => println!("{}", mv.uci()),
        None => print!("No moves!")
    }
}

fn main() {
    // best_move_of_input();
    play_vs_self(5);

    // let mut board = Board::from_fen("r1b1kbnr/pppp1ppp/2n5/4p3/3P4/2N1Bq2/PPP1PPPP/R2QKB1R w KQkq - 0 5").unwrap();
    // for mv in board.get_legal_moves() {
    //     println!("{}", mv.uci());
    // }

    // uci::setup_uci_engine();
}

// start
// rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1