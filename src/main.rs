mod chess;
mod zobrist;
mod prng;
mod engine;
mod uci;

mod bchess;
mod bengine;

use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::time::Instant;
use std::sync::{LazyLock, Once, OnceLock};

use crate::bchess::board::make_move;
use crate::bchess::magic_tables;
use crate::chess::{Board, BoardState};
use crate::engine::SearchOptions;
use crate::uci::run_uci_mode;
use crate::zobrist::ZobristHasher;

fn play_vs_self(board: &mut Board, options: &SearchOptions) {
    while board.is_live() {
        match engine::search(board, options.clone(), None, None).expect("No halts = no Err") {
            Some(mv) => {
                println!("{}", mv.uci());
                board.make_move(&mv, false);
                println!("{}", board);
                println!("{}", board.get_fen());
            },
            None => {
                break;
            }
        }
    }
    match board.get_state() {
        BoardState::WhiteWin => println!("white wins!"),
        BoardState::BlackWin => println!("black wins!"),
        BoardState::Stalemate => println!("stalemate"),
        BoardState::ThreefoldRepetition => println!("threefold repetition"),
        BoardState::FiftyMoveRule => println!("fifty move rule"),
        BoardState::InsufficientMaterial => println!("insufficient material"),
        BoardState::Live => unreachable!()
    };
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

pub static ZOBRIST_HASHER: OnceLock<ZobristHasher> = OnceLock::new();

fn init_statics() {
    ZOBRIST_HASHER.set(ZobristHasher::new(234234543)).map_err(|_| ()).expect("error initializing zobrist hash");
    magic_tables::init_tables();
}

fn main() {
    init_statics();

    let fen = "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1";

    let bboard = bchess::board::Board::new(fen).unwrap();
    // bchess::board::gen_legal_moves(&bboard, &mut Vec::new()); // init magic

    let start = Instant::now();
    bchess::board::gen_legal_moves(&bboard, &mut Vec::new());
    let elapsed = start.elapsed();

    println!("bitboard time: {:?}", elapsed);

    ///////////////////////////

    let mut board = Board::new(fen).unwrap();

    let start = Instant::now();
    board.get_legal_moves();
    let elapsed = start.elapsed();

    println!("mailbox time: {:?}", elapsed);

    // best_move_of_input(options.clone());

    // play_vs_self(&mut board, &options);

    // run_uci_mode();

    // let mut board = bchess::board::Board::default();
    // // dbg!(board);

    // board = bchess::board::make_move(&board, bchess::mv::Move::from_uci("e2e4", &board).unwrap());
    // board = bchess::board::make_move(&board, bchess::mv::Move::from_uci("g8f6", &board).unwrap());
    // board = bchess::board::make_move(&board, bchess::mv::Move::from_uci("e4e5", &board).unwrap());

    // let mut v = Vec::new();
    // bchess::board::gen_legal_moves(&board, &mut v);
    // for mv in &v {
    //     println!("{}", mv.uci());
    // }

    // board = bchess::board::make_move(&board, bchess::mv::Move::from_uci("d7d5", &board).unwrap());
    // board = bchess::board::make_move(&board, bchess::mv::Move::from_uci("e5d6", &board).unwrap());
    // board = bchess::board::make_move(&board, bchess::mv::Move::from_uci("e7e6", &board).unwrap());
    // board = bchess::board::make_move(&board, bchess::mv::Move::from_uci("d6c7", &board).unwrap());
    // board = bchess::board::make_move(&board, bchess::mv::Move::from_uci("f8e7", &board).unwrap());
    // board = bchess::board::make_move(&board, bchess::mv::Move::from_uci("c7b8Q", &board).unwrap());
    // board = bchess::board::make_move(&board, bchess::mv::Move::from_uci("e8g8", &board).unwrap());
    // println!("{}", board)
}

// r1bqk2r/1ppp1ppp/5n2/p3p3/1bQnP3/3B3N/PPPP1PPP/RNB1K2R b KQkq - 3 7

// start
// rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1