pub mod piece; // pub mod PIECE...
pub mod coord; // pub mod COORD...
pub mod mv; // pub mod MOVE...
pub mod board; // pub mod BOARD!!!

use crate::piece::{Piece, PieceType};
use crate::board::Board;
use crate::coord::Coord;
use crate::mv::Move;

use std::time::Instant;

fn score_side(board: &Board, color: bool) -> f64 {
    let mut score = Coord::all().map(|c| {
        if board.square_is_color(c, color) {
            match board.get_square(c).unwrap().piece_type {
                PieceType::Rook => 5000.0,
                PieceType::Knight => 3000.0,
                PieceType::Bishop => 3000.0,
                PieceType::Queen => 9000.0,
                PieceType::King => 0.0,
                PieceType::Pawn => 1000.0
            }
        } else {
            0.0
        }
    }).sum::<f64>();

    for x in 0..8 {
        let num_pawns = Coord::file(x).filter(|&c| {
            board.square_is_piece(c, color, PieceType::Pawn)
        }).count();
        if num_pawns > 1 {
            score -= (num_pawns * 20) as f64;
        }
    }
    score
}

fn relative_score(board: &Board) -> f64 {
    score_side(board, board.get_side_to_move()) - score_side(board, !board.get_side_to_move())
}

fn negamax(board: &mut Board, depth: usize, mut alpha: f64, beta: f64) -> f64 {
    if depth == 0 {
        return relative_score(board);
    }
    let opts = board.get_legal_moves();
    if opts.len() == 0 {
        return if board.is_check() {
            f64::MIN
        } else {
            0.0
        };
    }
    let mut max = f64::MIN;
    for mv in opts {
        board.make_move(&mv, true);
        let score = -negamax(board, depth - 1, -2.0 * beta, -2.0 * alpha) * 0.5;
        board.undo_move(&mv);
        if score > max {
            max = score;
            if max > alpha {
                alpha = score;
                if alpha >= beta {
                    break;
                }
            }
        }
    }
    max
}

fn find_best_move(board: &mut Board, max_depth: usize) -> Option<Move> {
    let mut best_move = None;
    let mut best_score = f64::MIN;
    for mv in board.get_legal_moves() {
        board.make_move(&mv, true);
        let score = -negamax(board, max_depth - 1, f64::MIN, f64::MAX);
        board.undo_move(&mv);
        if score == f64::MAX {
            return Some(mv);
        }
        if score > best_score {
            best_score = score;
            best_move = Some(mv);
        }
    }
    best_move
}

fn play_vs_self(depth: usize) {
    let mut board = Board::default();
    loop {
        match find_best_move(&mut board, depth) {
            Some(mv) => {
                println!("{}", mv.uci());
                board.make_move(&mv, false);
                println!("{}", board);
            },
            None => {
                println!("ggs");
                return;
            }
        }
        if board.fifty_move_rule() {
            println!("ggs (50 move rule)");
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
        .parse::<usize>() else { panic!("Error: not a natural number"); };
    if depth == 0 { panic!("Not zero idiot"); }

    let start = Instant::now();

    let best_move = find_best_move(&mut board, depth);

    println!("Time: {:?}", start.elapsed());

    match best_move {
        Some(mv) => println!("{}", mv.uci()),
        None => print!("No moves!")
    }
}

fn main() {
    // best_move_of_input();
    play_vs_self(5);
}