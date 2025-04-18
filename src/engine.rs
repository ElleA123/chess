use crate::piece::PieceType;
use crate::board::Board;
use crate::coord::Coord;
use crate::mv::Move;

pub fn get_best_move(board: &mut Board, max_depth: u32) -> Option<Move> {
    let mut best_move = None;
    let mut best_score = f64::MIN;
    for mv in board.get_legal_moves() {
        board.make_move(&mv, true);
        let score = -negamax(board, max_depth - 1, f64::MIN, f64::MAX);
        board.undo_move();
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

fn negamax(board: &mut Board, depth: u32, mut alpha: f64, beta: f64) -> f64 {
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
        board.undo_move();
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

fn relative_score(board: &Board) -> f64 {
    score_side(board, board.get_side_to_move()) - score_side(board, !board.get_side_to_move())
}

fn score_side(board: &Board, color: bool) -> f64 {
    let mut score = Coord::ALL.iter().map(|c| {
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
        let num_pawns = Coord::file(x).iter().filter(|c| {
            board.square_is_piece(*c, color, PieceType::Pawn)
        }).count();
        if num_pawns > 1 {
            score -= (num_pawns * 20) as f64;
        }
    }
    score
}