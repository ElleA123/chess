use crate::ZOBRIST_HASHER;

use super::piece::{Color, PieceType, Piece};
use super::mv::{Move, MoveType};
use super::coord::{Coord, COORDS};

pub const START_POS_FEN: &'static str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

#[derive(Debug, Clone, Copy)]
pub struct Castles {
    pub w_k: bool,
    pub w_q: bool,
    pub b_k: bool,
    pub b_q: bool
}

pub const CASTLE_W_K: Move = Move { from: Coord::new(7, 4), to: Coord::new(7, 6), move_type: MoveType::Castle };
pub const CASTLE_W_Q: Move = Move { from: Coord::new(7, 4), to: Coord::new(7, 2), move_type: MoveType::Castle };
pub const CASTLE_B_K: Move = Move { from: Coord::new(0, 4), to: Coord::new(0, 6), move_type: MoveType::Castle };
pub const CASTLE_B_Q: Move = Move { from: Coord::new(0, 4), to: Coord::new(0, 2), move_type: MoveType::Castle };

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BoardState {
    Live,
    WhiteWin,
    BlackWin,
    Stalemate,
    ThreefoldRepetition,
    FiftyMoveRule,
    InsufficientMaterial
}

#[derive(Debug)]
struct UndoData {
    mv: Move,
    captured: Option<Piece>,
    en_passant: Option<Coord>,
    allowed_castling: Castles,
    halfmove_count: u32,
}

#[derive(Debug)]
pub struct Board {
    board: [[Option<Piece>; 8]; 8],
    side_to_move: Color,
    allowed_castling: Castles,
    en_passant: Option<Coord>,
    halfmove_count: u32,
    fullmove_num: u32,
    state: BoardState,
    undo_stack: Vec<UndoData>,
    history: Vec<u64>,
    // hasher: Arc<ZobristHasher> // theres probably a reason i should do this but idk it
}

const R_STEPS: [(isize, isize); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];
const N_STEPS: [(isize, isize); 8] = [(2, 1), (2, -1), (1, 2), (1, -2), (-1, 2), (-1, -2), (-2, 1), (-2, -1)];
const B_STEPS: [(isize, isize); 4] = [(1, 1), (1, -1), (-1, 1), (-1, -1)];
const KQ_STEPS: [(isize, isize); 8] = [(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (1, -1), (-1, 1), (-1, -1)];

impl std::fmt::Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::from("\n");
        for row in self.board {
            for cell in row {
                s += (match cell {
                    Some(p) => p.to_string(),
                    None => String::from(".")
                } + " ").as_str();
            }
            s += "\n";
        }
        write!(f, "{}", s)
    }
}

impl Board {
    fn make_position(fen: &str) -> Option<(
        [[Option<Piece>; 8]; 8], Color, Castles, Option<Coord>, u32, u32
    )> {
        if !fen.is_ascii() || fen.is_empty() { return None; }

        let [
            pieces, side_to_move, allowed_castling, en_passant, halfmove_count, fullmove_num
        ] = fen.split(" ").collect::<Vec<_>>().try_into().ok()?;

        // Position
        let mut board: [[Option<Piece>; 8]; 8] = [[None; 8]; 8];

        // TODO: check for repeated numbers (e.g. "44") in fen
        let mut y = 0;
        for rank in pieces.split("/") {
            if y >= 8 { return None; }

            let mut x = 0;
            for p in rank.chars() {
                if x >= 8 { return None; }

                if let Some(piece) = Piece::new(p) {
                    board[y][x] = Some(piece);
                    x += 1;
                }
                else if p.is_ascii_digit() && p != '0' {
                    x += p.to_digit(10).unwrap() as usize;
                }
                else {
                    return None;
                }
            }
            if x != 8 { return None; }
            y += 1;
        }
        if y != 8 { return None; }

        // Side to move
        let side_to_move = match side_to_move {
            "w" => Color::White,
            "b" => Color::Black,
            _ => return None
        };

        // Castling avilability - TODO: add error handling
        let allowed_castling = Castles {
            w_k: allowed_castling.contains("K"),
            w_q: allowed_castling.contains("Q"),
            b_k: allowed_castling.contains("k"),
            b_q: allowed_castling.contains("q"),
        };

        // En passant
        let en_passant = match en_passant {
            "-" => None,
            square => match Coord::from_san(square) {
                Some(c) => Some(c),
                None => { return None; }
            }
        };

        // Halfmove count
        let Ok(halfmove_count) = halfmove_count.parse::<u32>() else { return None; };
        // Fullmove num
        let Ok(fullmove_num) = fullmove_num.parse::<u32>() else { return None; };

        Some((board, side_to_move, allowed_castling, en_passant, halfmove_count, fullmove_num))
    }

    pub fn new(fen: &str) -> Option<Self> {
        Self::make_position(fen).map(
            |(board, side_to_move, allowed_castling, en_passant, halfmove_count, fullmove_num)|
            Self {
                board,
                side_to_move,
                allowed_castling,
                en_passant,
                halfmove_count,
                fullmove_num,
                state: BoardState::Live,
                undo_stack: Vec::new(),
                history: Vec::new(),
            }
        )
    }

    pub fn default() -> Self {
        Self::new(START_POS_FEN).unwrap()
    }

    pub fn set_position(&mut self, fen: &str) {
        // Set position without reallocating undo_stack and history.
        // Why not? :)
        let Some(
            (board, side_to_move, allowed_castling, en_passant, halfmove_count, fullmove_num)
        ) = Self::make_position(fen) else { return; };
        
        self.board = board;
        self.side_to_move = side_to_move;
        self.allowed_castling = allowed_castling;
        self.en_passant = en_passant;
        self.halfmove_count = halfmove_count;
        self.fullmove_num = fullmove_num;
        self.state = BoardState::Live;
        self.undo_stack.clear();
        self.history.clear();
    }

    pub fn get_fen(&self) -> String {
        let mut fen = String::with_capacity(90);
        // Board
        for y in 0..8 {
            let mut gap = 0;
            for x in 0..8 {
                match self.board[y][x] {
                    Some(p) => {
                        if gap > 0 {
                            fen += &gap.to_string();
                            gap = 0;
                        }
                        fen += &p.to_string();
                    },
                    None => gap += 1
                }
            }
            if gap > 0 {
                fen += &gap.to_string();
            }
            if y != 7 {
                fen += "/";
            }
        }

        // Side to move
        fen += if self.side_to_move.is_white() {" w "} else {" b "};

        // Castling
        let mut can_castle = false;
        if self.allowed_castling.w_k { fen += "K"; can_castle = true; }
        if self.allowed_castling.w_q { fen += "Q"; can_castle = true; }
        if self.allowed_castling.b_k { fen += "k"; can_castle = true; }
        if self.allowed_castling.b_q { fen += "q"; can_castle = true; }
        if !can_castle { fen += "-"; }
        fen += " ";

        // En passant
        match self.en_passant {
            Some(c) => fen += &c.to_string(),
            None => fen += "-"
        };
        fen += " ";

        // Halfmove count & fullmove number
        fen += &self.halfmove_count.to_string();
        fen += " ";
        fen += &self.fullmove_num.to_string();

        return fen;
    }

    pub const fn get_board(&self) -> &[[Option<Piece>; 8]; 8] {
        &self.board
        // let mut board = [[None; 8]; 8];
        // for y in 0..8 {
        //     for x in 0..8 {
        //         board[y][x] = self.board[y][x].as_ref();
        //     }
        // }
        // board
    }

    pub const fn get_square(&self, coord: Coord) -> Option<Piece> {
        self.board[coord.y][coord.x]
    }

    pub fn square_is_color(&self, coord: Coord, color: Color) -> bool {
        match self.get_square(coord) {
            Some(piece) => piece.color == color,
            None => false
        }
    }

    pub fn square_is_piece_type(&self, coord: Coord, piece_type: PieceType) -> bool {
        match self.get_square(coord) {
            Some(piece) => piece.piece_type == piece_type,
            None => false
        }
    }

    // pub fn square_is_piece(&self, coord: Coord, piece: &Piece) -> bool {
    //     self.square_is_color(coord, piece.color) && self.square_is_piece_type(coord, piece.piece_type)
    // }

    pub const fn get_side_to_move(&self) -> Color {
        self.side_to_move
    }

    pub const fn get_allowed_castling(&self) -> Castles {
        self.allowed_castling
    }

    pub const fn get_en_passant(&self) -> Option<Coord> {
        self.en_passant
    }

    pub const fn get_state(&self) -> BoardState {
        self.state
    }

    pub const fn is_live(&self) -> bool {
        match self.state {
            BoardState::Live => true,
            _ => false
        }
    }

    pub fn make_move(&mut self, mv: &Move, undoable: bool) {
        if !self.is_live() { return; }
        // Only legal moves should make it to this function
        let Coord { y: from_y, x: from_x } = mv.from;
        let Coord { y: to_y, x: to_x } = mv.to;
        let piece = self.board[from_y][from_x].unwrap();

        let captured = self.board[to_y][to_x];
        let is_capture = captured.is_some() || mv.move_type == MoveType::EnPassant;

        // Add data to undo this move, or remove old undo data
        if undoable {
            self.undo_stack.push(UndoData {
                mv: mv.clone(),
                captured,
                en_passant: self.en_passant,
                allowed_castling: self.allowed_castling,
                halfmove_count: self.halfmove_count
            });
        } else {
            self.undo_stack.clear();
        }

        // Make the swap
        self.board[to_y][to_x] = if let MoveType::Promotion(pt) = mv.move_type {
            Some(Piece {
                piece_type: pt,
                color: piece.color,
            })
        } else {
            Some(piece)
        };
        self.board[from_y][from_x] = None;

        // En Passant
        if mv.move_type == MoveType::EnPassant {
            self.board[from_y][to_x] = None;
        }

        // Castling
        if mv.move_type == MoveType::Castle {
            let f_x = (to_x * 7 - 14) / 4;
            let t_x = (from_x + to_x) / 2;

            let extra_piece = self.board[from_y][f_x].unwrap();
            self.board[to_y][t_x] = Some(extra_piece);
            self.board[from_y][f_x] = None;
        }

        // Update castling availability -- a bit inefficient but like whatevs?
        match (from_y, from_x) {
            (7, 4) => {
                self.allowed_castling.w_k = false;
                self.allowed_castling.w_q = false;
            },
            (0, 4) => {
                self.allowed_castling.b_k = false;
                self.allowed_castling.b_q = false;
            },
            (7, 7) => { self.allowed_castling.w_k = false; },
            (7, 0) => { self.allowed_castling.w_q = false; },
            (0, 7) => { self.allowed_castling.b_k = false; },
            (0, 0) => { self.allowed_castling.b_q = false; },
            _ => ()
        };

        // Update en passant square
        if piece.piece_type == PieceType::Pawn && to_y.abs_diff(from_y) == 2 {
            self.en_passant = Some(Coord::new(match piece.color {
                Color::White => to_y + 1,
                Color::Black => to_y - 1,
            }, to_x));
        } else {
            self.en_passant = None;
        }

        // Update fullmove num after black moves
        if self.side_to_move.is_black() {self.fullmove_num += 1;}
        // Update turn
        self.side_to_move = !self.side_to_move;

        // Update halfmove count
        if piece.piece_type == PieceType::Pawn || is_capture {
            self.halfmove_count = 0;
        } else {
            self.halfmove_count += 1;
        }

        // Update board state (EXCEPT if stalemate, which is weird but seems fast)
        self.update_state_post_move();

        // Log new position in history
        self.history.push(ZOBRIST_HASHER.hash(self));
    }

    pub fn undo_move(&mut self) {
        let Some(undo_data) = self.undo_stack.pop() else {return};

        let Move { from: Coord { y: from_y, x: from_x }, to: Coord { y: to_y, x: to_x }, move_type } = undo_data.mv;

        let piece = self.board[to_y][to_x].unwrap();

        // Delete current position from history
        self.history.pop();

        // Swap
        self.board[from_y][from_x] = if let MoveType::Promotion(_) = move_type {
            Some(Piece {
                piece_type: PieceType::Pawn,
                color: piece.color
            })
        } else {
            Some(piece)
        };
        self.board[to_y][to_x] = undo_data.captured;

        if move_type == MoveType::EnPassant {
            self.board[from_y][to_x] = Some(Piece {
                piece_type: PieceType::Pawn,
                color: self.side_to_move
            });
        }

        if move_type == MoveType::Castle {
            let (f_x, t_x) = if to_x == 6 {(7, 5)} else {(0, 3)};
            let extra_piece = self.board[to_y][t_x].unwrap();
            self.board[from_y][f_x] = Some(extra_piece);
            self.board[to_y][t_x] = None;
        }

        // Update values from saved data
        self.allowed_castling = undo_data.allowed_castling;
        self.en_passant = undo_data.en_passant;
        self.halfmove_count = undo_data.halfmove_count;
        
        // Undo fullmove count
        if self.side_to_move.is_white() {
            self.fullmove_num -= 1;
        }
        // Update turn
        self.side_to_move = !self.side_to_move;

        // Reset board state
        self.state = BoardState::Live;
    }

    pub fn get_legal_moves(&mut self) -> Vec<Move> {
        if !self.is_live() { return Vec::new(); }

        let mut moves = Vec::with_capacity(80);
        let piece_coords: Vec<Coord> = self.find_players_pieces(self.side_to_move).collect();
        for coord in piece_coords {
            self.get_piece_moves(coord, &mut moves);
        }
        if moves.is_empty() {
            self.update_state_no_moves();
        }
        moves
    }

    pub fn find_players_pieces<'a>(&'a self, color: Color) -> impl Iterator<Item = Coord> + 'a {
        COORDS.into_iter().filter(move |&c| self.square_is_color(c, color))
    }

    fn get_piece_moves(&mut self, coord: Coord, moves: &mut Vec<Move>) {
        let piece = self.get_square(coord).unwrap();
        match piece.piece_type {
            PieceType::Rook => self.get_rook_moves(coord, moves),
            PieceType::Knight => self.get_knight_moves(coord, moves),
            PieceType::Bishop => self.get_bishop_moves(coord, moves),
            PieceType::Queen => self.get_queen_moves(coord, moves),
            PieceType::King => self.get_king_moves(coord, moves),
            PieceType::Pawn => self.get_pawn_moves(coord, moves),
        }
    }

    fn get_linear_moves(&mut self, coord: Coord, step_list: &[(isize, isize)], one_step_only: bool, moves: &mut Vec<Move>) {
        let color = self.get_square(coord).unwrap().color;
        for &step in step_list {
            let mut test_coord = coord;
            while test_coord.add(step) {
                if self.square_is_color(test_coord, color) { break; }
                
                let mv = Move::new(coord, test_coord, MoveType::Basic);
                if self.move_is_legal(&mv) { moves.push(mv); }

                if self.square_is_color(test_coord, !color) { break; }

                if one_step_only { break; }
            }
        }
    }

    fn get_rook_moves(&mut self, coord: Coord, moves: &mut Vec<Move>) {
        self.get_linear_moves(coord, &R_STEPS, false, moves)
    }
    fn get_knight_moves(&mut self, coord: Coord, moves: &mut Vec<Move>) {
        self.get_linear_moves(coord, &N_STEPS, true, moves)
    }
    fn get_bishop_moves(&mut self, coord: Coord, moves: &mut Vec<Move>) {
        self.get_linear_moves(coord, &B_STEPS, false, moves)
    }
    fn get_queen_moves(&mut self, coord: Coord, moves: &mut Vec<Move>) {
        self.get_linear_moves(coord, &KQ_STEPS, false, moves)
    }

    fn get_king_moves(&mut self, coord: Coord, moves: &mut Vec<Move>) {
        self.get_linear_moves(coord, &KQ_STEPS, true, moves);

        // TODO: castling out of/through check
        // TODO: make four separate consts, or just write it out in this fn
        if coord.x == 4 && coord.y == 7 {
            if self.allowed_castling.w_k && self.board[7][5].is_none() && self.board[7][6].is_none() {
                if self.move_is_legal(&CASTLE_W_K) { moves.push(CASTLE_W_K); }
            }
            if self.allowed_castling.w_q && self.board[7][2].is_none() && self.board[7][3].is_none() && self.board[7][4].is_none() {
                if self.move_is_legal(&CASTLE_W_Q) { moves.push(CASTLE_W_Q); }
            }
        }
        if coord.x == 4 && coord.y == 0 {
            if self.allowed_castling.b_k && self.board[0][5].is_none() && self.board[0][6].is_none() {
                if self.move_is_legal(&CASTLE_B_K) { moves.push(CASTLE_B_K); }
            }
            if self.allowed_castling.b_q && self.board[0][2].is_none() && self.board[0][3].is_none() && self.board[0][4].is_none() {
                if self.move_is_legal(&CASTLE_B_Q) { moves.push(CASTLE_B_Q); }
            }
        }
    }

    fn get_pawn_moves(&mut self, coord: Coord, moves: &mut Vec<Move>) {
        let Coord { y, x } = coord;
        let color = self.board[y][x].unwrap().color;

        let pawn_dir = match color {
            Color::White => -1,
            Color::Black => 1
        };
        let will_promote = y == match color {
            Color::White => 1,
            Color::Black => 6
        };

        if self.board[(y as isize + pawn_dir) as usize][x].is_none() {
            // Forward 1
            if will_promote {
                let promos = Move::promotions(coord, Coord::new((y as isize + pawn_dir) as usize, x));
                if self.move_is_legal(&promos[0]) { moves.extend(promos); }
            } else {
                let mv = Move::new(coord, Coord::new((y as isize + pawn_dir) as usize, x), MoveType::Basic);
                if self.move_is_legal(&mv) { moves.push(mv); }
            }
            // Forward 2
            if (color.is_white() && y == 6 || color.is_black() && y == 1) && self.board[(y as isize + 2*pawn_dir) as usize][x].is_none() {
                let mv = Move::new(coord, Coord::new((y as isize + 2*pawn_dir) as usize, x), MoveType::Basic);
                if self.move_is_legal(&mv) { moves.push(mv); }
            }
        }

        if x != 0 {
            // Capture left
            if self.square_is_color(Coord::new((y as isize + pawn_dir) as usize, x - 1), !color) {
                if will_promote {
                    let promos = Move::promotions(coord, Coord::new((y as isize + pawn_dir) as usize, x - 1));
                    if self.move_is_legal(&promos[0]) { moves.extend(promos); }
                } else {
                    let mv = Move::new(coord, Coord::new((y as isize + pawn_dir) as usize, x - 1), MoveType::Basic);
                    if self.move_is_legal(&mv) { moves.push(mv); }
                }
            }
            // En passant left
            if let Some(sq) = self.en_passant {
                if sq.y == (y as isize + pawn_dir) as usize && sq.x == x - 1 {
                    let mv = Move::new(coord, Coord::new((y as isize + pawn_dir) as usize, x - 1), MoveType::EnPassant);
                    if self.move_is_legal(&mv) { moves.push(mv); }
                }
            }
        }
        if x != 7 {
            // Capture right
            if self.square_is_color(Coord::new((y as isize + pawn_dir) as usize, x + 1), !color) {
                if will_promote {
                    let promos = Move::promotions(coord, Coord::new((y as isize + pawn_dir) as usize, x + 1));
                    if self.move_is_legal(&promos[0]) { moves.extend(promos); }
                } else {
                    let mv = Move::new(coord, Coord::new((y as isize + pawn_dir) as usize, x + 1), MoveType::Basic);
                    if self.move_is_legal(&mv) { moves.push(mv); }
                }
            }
            // En passant right
            if let Some(sq) = self.en_passant {
                if sq.y == (y as isize + pawn_dir) as usize && sq.x == x + 1 {
                    let mv = Move::new(coord, Coord::new((y as isize + pawn_dir) as usize, x + 1), MoveType::EnPassant);
                    if self.move_is_legal(&mv) { moves.push(mv); }
                }
            }
        }
    }

    pub fn move_is_legal(&mut self, mv: &Move) -> bool {
        self.make_move(mv, true);
        let is_legal = !self.king_is_attacked(!self.side_to_move);
        self.undo_move();
        is_legal
    }

    fn king_is_attacked(&self, color: Color) -> bool {
        let king = COORDS.into_iter().find(|&c|
            self.square_is_color(c, color) && self.square_is_piece_type(c, PieceType::King)
        ).unwrap();

        self.square_is_attacked(king, !color)
    }

    fn square_is_attacked(&self, target: Coord, color: Color) -> bool {
        self.find_players_pieces(color).any(|coord| self.piece_attacks(coord, target))
    }

    fn piece_attacks(&self, coord: Coord, target: Coord) -> bool {
        let piece = self.get_square(coord).unwrap();
        match piece.piece_type {
            PieceType::Rook => {
                if coord.x != target.x && coord.y != target.y { return false; }
                self.can_linearly_attack(coord, target, &R_STEPS)
            },
            PieceType::Knight => {
                let x_diff = coord.x.abs_diff(target.x);
                let y_diff = coord.y.abs_diff(target.y);
                (x_diff == 2 && y_diff == 1) || (x_diff == 1 && y_diff == 2)
            },
            PieceType::Bishop => {
                if coord.x.abs_diff(target.x) != coord.y.abs_diff(target.y) { return false; }
                self.can_linearly_attack(coord, target, &B_STEPS)
            },
            PieceType::Queen => {
                if coord.x != target.x && coord.y != target.y
                    && coord.x.abs_diff(target.x) != coord.y.abs_diff(target.y) { return false; }
                self.can_linearly_attack(coord, target, &KQ_STEPS)
            },
            PieceType::King => {
                coord.x.abs_diff(target.x) <= 1 && coord.y.abs_diff(target.y) <= 1
            },
            PieceType::Pawn => {
                let dir = match self.get_square(coord).unwrap().color {
                    Color::White => -1,
                    Color::Black => 1
                };
                coord.x.abs_diff(target.x) == 1 && (coord.y as isize + dir) as usize == target.y
            },
        }
    }

    fn can_linearly_attack(&self, from: Coord, to: Coord, step_list: &[(isize, isize)]) -> bool {
        for &step in step_list {
            let mut test_coord = from;
            while test_coord.add(step) {
                if test_coord == to {
                    return true;
                }
                if self.get_square(test_coord).is_some() {
                    break;
                }
            }
        }
        return false;
    }

    fn update_state_no_moves(&mut self) {
        self.state = if !self.is_check() {
            BoardState::Stalemate
        }
        else { match self.side_to_move {
            Color::White => BoardState::BlackWin,
            Color::Black => BoardState::WhiteWin
        }};
    }

    pub fn is_check(&self) -> bool {
        self.king_is_attacked(self.side_to_move)
    }

    fn update_state_post_move(&mut self) {
        if self.halfmove_count >= 100 {
            self.state = BoardState::FiftyMoveRule;
        }
        else if self.check_threefold_repetition() {
            self.state = BoardState::ThreefoldRepetition;
        }
        else if self.check_insufficient_material() {
            self.state = BoardState::InsufficientMaterial;
        }
    }

    fn check_threefold_repetition(&self) -> bool {
        let Some(current) = self.history.last() else { return false; };

        let mut count = 0;
        for hash in self.history.iter().rev().step_by(2) {
            if hash == current {
                count += 1;
            }
            if count >= 3 {
                return true;
            }
        }
        return false;
    }

    fn check_insufficient_material(&self) -> bool {
        let mut w_knights = 0;
        let mut w_bishops = 0;
        let mut w_bishop_sq_color = 0;

        for coord in self.find_players_pieces(Color::White) {
            let piece_type = self.get_square(coord).unwrap().piece_type;
            match piece_type {
                PieceType::Rook => return false,
                PieceType::Queen => return false,
                PieceType::Pawn => return false,
                PieceType::Knight => w_knights += 1,
                PieceType::Bishop => {
                    w_bishops += 1;
                    w_bishop_sq_color = coord.idx() & 1;
                },
                PieceType::King => {}
            };

            if w_knights + w_bishops >= 2 {
                return false;
            }
        }   

        let mut b_knights = 0;
        let mut b_bishops = 0;
        let mut b_bishop_sq_color = 0;

        for coord in self.find_players_pieces(Color::Black) {
            let piece_type = self.get_square(coord).unwrap().piece_type;
            match piece_type {
                PieceType::Rook => return false,
                PieceType::Queen => return false,
                PieceType::Pawn => return false,
                PieceType::Knight => b_knights += 1,
                PieceType::Bishop => {
                    b_bishops += 1;
                    b_bishop_sq_color = coord.idx() & 1;
                },
                PieceType::King => {}
            };

            if b_knights + b_bishops >= 2 {
                return false;
            }

            if b_knights >= 1 && w_knights + w_bishops >= 1 {
                return false;
            }

            if b_bishops >= 1 && w_knights >= 1 {
                return false;
            }
        }

        if w_bishops == 1 && b_bishops == 1 && w_bishop_sq_color != b_bishop_sq_color {
            return false;
        }

        return true;
    }
}