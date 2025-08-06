mod chess;
mod zobrist;
mod engine;
mod uci;

use chess::Board;

use std::time::Instant;
use std::sync::LazyLock;

use crate::{
    chess::BoardState, engine::SearchOptions, uci::run_uci_mode, zobrist::ZobristHasher
};

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
    let Some(mut board) = Board::from_fen(fen.as_str()) else { panic!("invalid FEN"); };
    println!("{}", board);

    let start = Instant::now();

    let best_move = engine::search(&mut board, options, None, None).unwrap();

    println!("Time: {:?}", start.elapsed());

    match best_move {
        Some(mv) => println!("{}", mv.uci()),
        None => print!("No moves!")
    }
}

static ZOBRIST_HASHER: LazyLock<ZobristHasher> = LazyLock::new(|| ZobristHasher::new(234234543));

fn main() {
    // let mut board = Board::from_fen("r1bqkb1r/ppp1pppp/2n2n2/3p1Q2/4P3/8/PPPP1PPP/RNB1KBNR w KQkq d6 0 4").unwrap();
    // let mut board = Board::default();

    // let options = SearchOptions {
    //     max_depth: 5,
    //     time: usize::MAX,
    //     nodes: None,
    // };

    // let best_move = engine::search(&mut board, options).unwrap();
    // println!("{}", best_move.uci());

    // best_move_of_input(options.clone());

    // play_vs_self(&mut board, &options);

    run_uci_mode();
}

// r1bqk2r/1ppp1ppp/5n2/p3p3/1bQnP3/3B3N/PPPP1PPP/RNB1K2R b KQkq - 3 7

// start
// rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1