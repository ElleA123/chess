mod psts;

use crate::chess::*;

pub fn get_best_move(board: &mut Board, max_depth: usize) -> Option<Move> {
    let mut moves = board.get_legal_moves();

    sort_moves(board, &mut moves, 2);

    dfs_search(board, moves, max_depth)
}

pub fn sort_moves(board: &mut Board, moves: &mut Vec<Move>, depth: usize) {
    let scores: Vec<(Move, f64)> = moves.iter().map(|mv| {
        board.make_move(mv, true);
        let score = -negamax(board, depth - 1, f64::MIN, f64::MAX);
        board.undo_move();
        (mv.clone(), score)
    }).collect();
    
    moves.sort_by(|mv1, mv2|
        scores.iter().find(|(mv, _)| mv1 == mv).unwrap().1
        .total_cmp(&scores.iter().find(|(mv, _)| mv2 == mv).unwrap().1)
    );
}

pub fn dfs_search(board: &mut Board, moves: Vec<Move>, depth: usize) -> Option<Move> {
    let mut best_move = None;
    let mut best_score = f64::MIN;
    for mv in moves {
        board.make_move(&mv, true);
        let score = -negamax(board, depth - 1, f64::MIN, f64::MAX);
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

fn negamax(board: &mut Board, depth: usize, mut alpha: f64, beta: f64) -> f64 {
    if depth == 0 {
        return relative_score(board);
    }

    let moves = board.get_legal_moves();
    if moves.len() == 0 {
        return if board.is_check() {
            f64::MIN
        } else {
            0.0
        };
    }

    let mut max = f64::MIN;
    for mv in moves {
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

const MATERIAL_FACTOR: f64 = 1.0;
const PST_FACTOR: f64 = 0.01;

fn relative_score(board: &Board) -> f64 {
    score_side(board, board.get_side_to_move()) - score_side(board, !board.get_side_to_move())
}

fn score_side(board: &Board, color: Color) -> f64 {
    let mut score = 0.0;

    for coord in board.find_players_pieces(color) {
        let piece = board.get_square(coord).unwrap();
        score += MATERIAL_FACTOR * match piece.piece_type {
            PieceType::Rook => 5.,
            PieceType::Knight => 3.,
            PieceType::Bishop => 3.,
            PieceType::King => 0.,
            PieceType::Queen => 9.,
            PieceType::Pawn => 1.
        };
        score += PST_FACTOR * psts::get_mg(piece, coord) as f64;
    }

    score
}