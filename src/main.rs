mod chess;
mod engine;
mod prng;
mod uci;
mod zobrist;

use crate::chess::{make_move, Board};
use crate::engine::SearchOptions;
use crate::zobrist::ZobristHasher;
use crate::uci::run_uci_mode;

use std::time::Instant;

// fn play_vs_self(board: Option<Board>, options: SearchOptions) {
//     let mut board = board.unwrap_or_else(|| Board::default());

//     while board.is_live() {
//         match engine::search(board, options, None, None).expect("No halts = no Err") {
//             Some(mv) => {
//                 println!("{}", mv.uci());
//                 board = make_move(&board, mv);
//                 println!("{}", board);
//                 println!("{}", board.get_fen());
//             },
//             None => {
//                 break;
//             }
//         }
//     }
//     match board.get_state() {
//         BoardState::WhiteWin => println!("white wins!"),
//         BoardState::BlackWin => println!("black wins!"),
//         BoardState::Stalemate => println!("stalemate"),
//         BoardState::ThreefoldRepetition => println!("threefold repetition"),
//         BoardState::FiftyMoveRule => println!("fifty move rule"),
//         BoardState::InsufficientMaterial => println!("insufficient material"),
//         BoardState::Live => unreachable!()
//     };

//     println!("{}", board.get_fen());
// }

fn get_input(msg: &str) -> String {
    println!("{}", msg);
    let mut buf = String::new();
    std::io::stdin()
        .read_line(&mut buf)
        .expect("Failed to read line");
    buf.trim().to_owned()
}

fn best_move_of_input(options: SearchOptions) {
    let fen = get_input("Input FEN:");
    let Some(mut board) = Board::new(fen.as_str()) else { panic!("invalid FEN"); };
    println!("{}", board);

    let start = Instant::now();

    let best_move = engine::search(&mut board, options, None, None).unwrap();

    println!("Time: {:?}", start.elapsed());

    match best_move {
        Some(mv) => println!("{}", mv.uci()),
        None => print!("No moves!")
    }
}

pub static ZOBRIST_HASHER: ZobristHasher = ZobristHasher::new(234234543);

fn main() {
    chess::init_magic_tables();
    run_uci_mode();
}

// start
// rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1