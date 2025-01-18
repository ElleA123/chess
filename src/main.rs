pub mod coord;
pub mod board;
use crate::board::Board;
use crate::coord::Coord;

use std::time::Instant;

const PROMOTABLES: [PieceType; 4] = [PieceType::Rook, PieceType::Knight, PieceType::Bishop, PieceType::Queen];

#[derive(Debug, Clone, Copy, PartialEq)]
enum PieceType {
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
    Pawn
}

impl PieceType {
    fn from_char(c: char) -> Option<Self> {
        match c.to_ascii_lowercase() {
            'r' => Some(PieceType::Rook),
            'n' => Some(PieceType::Knight),
            'b' => Some(PieceType::Bishop),
            'q' => Some(PieceType::Queen),
            'k' => Some(PieceType::King),
            'p' => Some(PieceType::Pawn),
            _ => None
        }
    }

    fn to_string(&self) -> String {
        String::from(match self {
            &PieceType::Rook => 'r',
            &PieceType::Knight => 'n',
            &PieceType::Bishop => 'b',
            &PieceType::Queen => 'q',
            &PieceType::King => 'k',
            &PieceType::Pawn => 'p',
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Piece {
    piece_type: PieceType,
    color: bool
}

impl Piece {
    fn from_char(c: char) -> Option<Self> {
        if let Some(piece_type) = PieceType::from_char(c) {
            Some(Piece {
                piece_type,
                color: c.is_ascii_uppercase()
            })
        } else {
            None
        }
    }

    fn to_string(&self) -> String {
        if self.color {
            self.piece_type.to_string().to_ascii_uppercase()
        } else {
            self.piece_type.to_string()
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum MoveType {
    Basic,
    EnPassant,
    Castle,
    Promotion(PieceType)
}

#[derive(Debug, Clone, PartialEq)]
struct Move {
    from: Coord,
    to: Coord,
    move_type: MoveType
}

impl Move {
    fn new(from: Coord, to: Coord, move_type: MoveType) -> Self {
        Move {
            from,
            to,
            move_type
        }
    }

    fn promotions(from: Coord, to: Coord) -> impl Iterator<Item = Self> {
        PROMOTABLES.iter().map(move |&pt| Move {
            from,
            to,
            move_type: MoveType::Promotion(pt)
        })
    }

    fn uci(&self) -> String {
        let mut uci = format!("{}{}",
            self.from.to_string(),
            self.to.to_string()
        );
        if let MoveType::Promotion(pt) = self.move_type {
            uci += pt.to_string().as_str();
        }
        uci
    }
}


fn score_side(board: &Board, color: bool) -> f64 {
    let mut score = (0..64).map(|i| (i / 8, i % 8))
    .map(|(y, x)| {
        if board.square_is_color(y, x, color) {
            match board.get_square(y, x).unwrap().piece_type {
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
        let num_pawns = (0..8).into_iter().filter(|&y| {
            board.square_is_piece(y, x, color, PieceType::Pawn)
        }).count();
        if num_pawns > 1 {
            score -= (num_pawns * 20) as f64;
        }
    }
    score
}

fn score_board(board: &Board) -> f64 {
    score_side(board, board.get_side_to_move()) - score_side(board, !board.get_side_to_move())
}

fn negamax(board: &mut Board, depth: usize, mut alpha: f64, beta: f64) -> f64 {
    if depth == 0 {
        return score_board(board);
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
        board.make_move(&mv);
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
        board.make_move(&mv);
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
                board.make_move(&mv);
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
    best_move_of_input();
    // play_vs_self(4, 10.0);
}