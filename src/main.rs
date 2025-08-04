mod chess;
mod zobrist;
mod engine;
mod uci;

use chess::Board;
use engine::get_best_move;

use std::time::Instant;
use std::sync::LazyLock;

use crate::{
    chess::BoardState,
    zobrist::ZobristHasher
};

fn play_vs_self(board: &mut Board, depth: usize) {
    while board.is_live() {
        match get_best_move(board, depth) {
            Some(mv) => {
                println!("{}", mv.uci());
                board.make_move(&mv, false);
                println!("{}", board);
                // println!("{:?}", board);
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

static ZOBRIST_HASHER: LazyLock<ZobristHasher> = LazyLock::new(|| ZobristHasher::new(234234543));

fn main() {
    let mut board = Board::default();

    // board.make_move(&Move::from_uci("f5f2", &board).unwrap(), false);
    // println!("f5f2\n{}", board);

    // for mv in board.get_legal_moves() {
    //     println!("{:?}", mv);
    // }

    play_vs_self(&mut board, 5);

    // let next_mv = Move { from: Coord { y: 4, x: 3 }, to: Coord { y: 6, x: 4 }, move_type: MoveType::Basic };
    // println!("{}", board.get_legal_moves().contains(&next_mv));

    // board.make_move(, true);
    // println!("{}", board);
    // board.make_move(&Move { from: Coord { y: 4, x: 6 }, to: Coord { y: 3, x: 6 }, move_type: MoveType::Basic }, true);
    // println!("{}", board);
    // run_uci_mode();

    // best_move_of_input();
    // play_vs_self(&mut board, 5);

    // let mut board = Board::from_fen("r1b1kbnr/pppp1ppp/2n5/4p3/3P4/2N1Bq2/PPP1PPPP/R2QKB1R w KQkq - 0 5").unwrap();
    // for mv in board.get_legal_moves() {
    //     println!("{}", mv.uci());
    // }
}

// start
// rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1

// 1q6/1p4pk/3p3p/2p1pr2/2P2n2/2P1BN1P/1P3PP1/3R1K2 b - - 0 1

/*
undo_stack: [
UndoData {
    mv: Move { from: Coord { y: 6, x: 6 }, to: Coord { y: 4, x: 6 }, move_type: Basic },
    captured: None,
    en_passant: None,
    allowed_castling: Castles { w_k: false, w_q: false, b_k: false, b_q: false },
    halfmove_count: 6
},
UndoData { mv: Move { from: Coord { y: 4, x: 3 }, to: Coord { y: 6, x: 4 }, move_type: Basic }, captured: None, en_passant: Some(Coord { y: 5, x: 6 }), allowed_castling: Castles { w_k: false, w_q: false, b_k: false, b_q: false }, halfmove_count: 0 },
UndoData { mv: Move { from: Coord { y: 6, x: 6 }, to: Coord { y: 7, x: 5 }, move_type: Promotion(Rook) }, captured: Some(Piece { piece_type: King, color: White }), en_passant: None, allowed_castling: Castles { w_k: false, w_q: false, b_k: false, b_q: false }, halfmove_count: 1 },
UndoData { mv: Move { from: Coord { y: 4, x: 6 }, to: Coord { y: 3, x: 6 }, move_type: Basic }, captured: None, en_passant: None, allowed_castling: Castles { w_k: false, w_q: false, b_k: false, b_q: false }, halfmove_count: 0 }
]
*/