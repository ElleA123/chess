use crate::{Piece, PieceType};
use crate::mv::{Move, MoveType, CASTLES};
use crate::coord::Coord;

#[derive(Debug, Clone, Copy)]
pub enum BoardState {
    Live,
    WhiteWin,
    BlackWin,
    Stalemate,
    ThreefoldRepetition,
    FiftyMoveRule,
    InsufficientMaterial
}

struct UndoData {
    from: Coord,
    to: Coord,
    move_type: MoveType,
    captured: Option<Piece>,
    en_passant: Option<Coord>,
    allowed_castling: (bool, bool, bool, bool),
    halfmove_count: u32,
}

pub struct Board {
    board: [[Option<Piece>; 8]; 8],
    side_to_move: bool,
    allowed_castling: (bool, bool, bool, bool), // KQkq
    en_passant: Option<Coord>,
    halfmove_count: u32,
    fullmove_num: u32,
    undo_stack: Vec<UndoData>,
    state: BoardState,
}

const WHITE: bool = true;
const BLACK: bool = false;

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
    pub fn from_fen(fen: &str) -> Option<Self> {
        // TODO: reject non-ascii and check bytes
        let mut fen_fields = fen.split(" ");

        // Position
        let Some(pieces) = fen_fields.next() else { return None; };
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

        // Player to move
        let Some(side_to_move) = fen_fields.next() else { return None; };
        let side_to_move = match side_to_move {
            "w" => WHITE,
            "b" => BLACK,
            _ => return None
        };

        // Castling avilability - TODO: add error handling
        let Some(allowed_castling) = fen_fields.next() else { return None; };
        let allowed_castling = (
            allowed_castling.contains("K"),
            allowed_castling.contains("Q"),
            allowed_castling.contains("k"),
            allowed_castling.contains("q"),
        );

        // En passant
        let Some(en_passant) = fen_fields.next() else { return None; };
        let en_passant = match en_passant {
            "-" => None,
            square => match Coord::from_san(square) {
                Some(c) => Some(c),
                None => { return None; }
            }
        };

        let Some(halfmove_count) = fen_fields.next() else { return None; };
        let Ok(halfmove_count) = halfmove_count.parse::<u32>() else { return None; };

        let Some(fullmove_num) = fen_fields.next() else { return None; };
        let Ok(fullmove_num) = fullmove_num.parse::<u32>() else { return None; };

        if fen_fields.count() == 0 {
            Some(Board {
                board,
                side_to_move,
                allowed_castling,
                en_passant,
                halfmove_count,
                fullmove_num,
                undo_stack: Vec::with_capacity(8),
                state: BoardState::Live,
            })
        } else {
            None
        }
    }

    pub fn default() -> Self {
        Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap()
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
        fen += if self.side_to_move == WHITE {" w "} else {" b "};

        // Castling
        let mut can_castle = false;
        if self.allowed_castling.0 { fen += "K"; can_castle = true; }
        if self.allowed_castling.1 { fen += "Q"; can_castle = true; }
        if self.allowed_castling.2 { fen += "k"; can_castle = true; }
        if self.allowed_castling.3 { fen += "q"; can_castle = true; }
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

    pub fn get_board(&self) -> [[Option<&Piece>; 8]; 8] {
        let mut board = [[None; 8]; 8];
        for y in 0..8 {
            for x in 0..8 {
                board[y][x] = self.board[y][x].as_ref();
            }
        }
        board
    }

    pub const fn get_square(&self, coord: &Coord) -> Option<&Piece> {
        self.board[coord.y][coord.x].as_ref()
    }

    pub const fn square_is_color(&self, coord: &Coord, color: bool) -> bool {
        match self.get_square(coord) {
            Some(piece) => piece.color == color,
            None => false
        }
    }

    pub const fn square_is_piece_type(&self, coord: &Coord, piece_type: PieceType) -> bool {
        match self.get_square(coord) {
            Some(piece) => piece.piece_type as u8 == piece_type as u8,
            None => false
        }
    }

    pub const fn square_is_piece(&self, coord: &Coord, color: bool, piece_type: PieceType) -> bool {
        self.square_is_color(coord, color) && self.square_is_piece_type(coord, piece_type)
    }

    pub const fn get_side_to_move(&self) -> bool {
        self.side_to_move
    }

    pub const fn get_allowed_castling(&self) -> &(bool, bool, bool, bool) {
        &self.allowed_castling
    }

    pub const fn get_en_passant(&self) -> Option<&Coord> {
        self.en_passant.as_ref()
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
        let (from_y, from_x) = mv.get_from().vals();
        let (to_y, to_x) = mv.get_to().vals();
        let piece = self.board[from_y][from_x].unwrap();

        let captured = self.board[to_y][to_x];
        let is_capture = captured.is_some() || *mv.get_move_type() == MoveType::EnPassant;

        // Add data to undo this move
        if undoable {
            self.undo_stack.push(UndoData {
                from: *mv.get_from(),
                to: *mv.get_to(),
                move_type: *mv.get_move_type(),
                captured,
                en_passant: self.en_passant,
                allowed_castling: self.allowed_castling,
                halfmove_count: self.halfmove_count
            });
        } else {
            self.undo_stack.clear();
        }

        // Make the swap
        self.board[to_y][to_x] = if let MoveType::Promotion(pt) = mv.get_move_type() {
            Some(Piece {
                piece_type: *pt,
                color: piece.color,
            })
        } else {
            Some(piece)
        };
        self.board[from_y][from_x] = None;

        // En Passant
        if *mv.get_move_type() == MoveType::EnPassant {
            self.board[from_y][to_x] = None;
        }

        // Castling
        if *mv.get_move_type() == MoveType::Castle {
            let f_x = (to_x * 7 - 14) / 4;
            let t_x = (from_x + to_x) / 2;

            let extra_piece = self.board[from_y][f_x].unwrap();
            self.board[to_y][t_x] = Some(extra_piece);
            self.board[from_y][f_x] = None;
        }

        // Update castling availability -- a bit inefficient but like whatevs?
        match (from_y, from_x) {
            (7, 4) => { // K
                self.allowed_castling.0 = false;
                self.allowed_castling.1 = false;
            },
            (0, 4) => { // k
                self.allowed_castling.2 = false;
                self.allowed_castling.3 = false;
            },
            (7, 7) => { self.allowed_castling.0 = false; }, // qR
            (7, 0) => { self.allowed_castling.1 = false; }, // kR
            (0, 7) => { self.allowed_castling.2 = false; }, // qr
            (0, 0) => { self.allowed_castling.3 = false; }, // kr
            _ => ()
        };

        // Update en passant square
        if piece.piece_type == PieceType::Pawn && to_y.abs_diff(from_y) == 2 {
            self.en_passant = Some(Coord::new(if piece.color == WHITE {to_y + 1} else {to_y - 1}, to_x));
        } else {
            self.en_passant = None;
        }

        // Update fullmove num after black moves
        if !self.side_to_move {self.fullmove_num += 1;}
        // Update turn
        self.side_to_move = !self.side_to_move;

        // Update halfmove count
        if piece.piece_type == PieceType::Pawn || is_capture {
            self.halfmove_count = 0;
        } else {
            self.halfmove_count += 1;
        }
    }

    pub fn undo_move(&mut self) {
        let Some(undo_data) = self.undo_stack.pop() else {return};

        let (from_y, from_x) = undo_data.from.vals();
        let (to_y, to_x) = undo_data.to.vals();
        let piece = self.board[to_y][to_x].unwrap();

        // Swap
        self.board[from_y][from_x] = if let MoveType::Promotion(_) = undo_data.move_type {
            Some(Piece {
                piece_type: PieceType::Pawn,
                color: piece.color
            })
        } else {
            Some(piece)
        };
        self.board[to_y][to_x] = undo_data.captured;

        if undo_data.move_type == MoveType::EnPassant {
            self.board[from_y][to_x] = Some(Piece {
                piece_type: PieceType::Pawn,
                color: self.side_to_move
            });
        }

        if undo_data.move_type == MoveType::Castle {
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
        if self.side_to_move {
            self.fullmove_num -= 1;
        }
        // Update turn
        self.side_to_move = !self.side_to_move;
    }

    pub fn get_legal_moves(&mut self) -> Vec<Move> {
        let mut moves = Vec::with_capacity(40);
        let piece_coords: Vec<Coord> = self.find_players_pieces(self.side_to_move).collect();
        for coord in piece_coords {
            self.get_piece_moves(&coord, &mut moves);
        }
        if moves.is_empty() {
            self.update_state_no_moves();
        }
        moves
    }

    pub fn find_players_pieces<'a>(&'a self, color: bool) -> impl Iterator<Item = Coord> + 'a {
        Coord::ALL.into_iter().filter(move |c| self.square_is_color(c, color))
    }

    fn get_piece_moves(&mut self, coord: &Coord, moves: &mut Vec<Move>) {
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

    fn get_linear_moves(&mut self, coord: &Coord, step_list: &[(isize, isize)], one_step_only: bool, moves: &mut Vec<Move>) {
        let color = self.get_square(coord).unwrap().color;
        for step in step_list {
            let mut test_coord = *coord;
            while test_coord.add(step) {
                if self.square_is_color(&test_coord, color) { break; }
                
                let mv = Move::new(*coord, test_coord, MoveType::Basic);
                if self.move_is_legal(&mv) { moves.push(mv); }

                if self.square_is_color(&test_coord, !color) { break; }

                if one_step_only { break; }
            }
        }
    }

    fn get_rook_moves(&mut self, coord: &Coord, moves: &mut Vec<Move>) {
        self.get_linear_moves(coord, &R_STEPS, false, moves)
    }
    fn get_knight_moves(&mut self, coord: &Coord, moves: &mut Vec<Move>) {
        self.get_linear_moves(coord, &N_STEPS, true, moves)
    }
    fn get_bishop_moves(&mut self, coord: &Coord, moves: &mut Vec<Move>) {
        self.get_linear_moves(coord, &B_STEPS, false, moves)
    }
    fn get_queen_moves(&mut self, coord: &Coord, moves: &mut Vec<Move>) {
        self.get_linear_moves(coord, &KQ_STEPS, false, moves)
    }

    fn get_king_moves(&mut self, coord: &Coord, moves: &mut Vec<Move>) {
        self.get_linear_moves(coord, &KQ_STEPS, true, moves);

        // TODO: castling out of/through check
        if coord.x == 4 && coord.y == 7 {
            if self.allowed_castling.0 && self.board[7][5].is_none() && self.board[7][6].is_none() {
                if self.move_is_legal(&CASTLES[0]) { moves.push(CASTLES[0].clone()); }
            }
            if self.allowed_castling.1 && self.board[7][2].is_none() && self.board[7][3].is_none() && self.board[7][4].is_none() {
                if self.move_is_legal(&CASTLES[1]) { moves.push(CASTLES[1].clone()); }
            }
        }
        if coord.x == 4 && coord.y == 0 {
            if self.allowed_castling.2 && self.board[0][5].is_none() && self.board[0][6].is_none() {
                if self.move_is_legal(&CASTLES[2]) { moves.push(CASTLES[2].clone()); }
            }
            if self.allowed_castling.3 && self.board[0][2].is_none() && self.board[0][3].is_none() && self.board[0][4].is_none() {
                if self.move_is_legal(&CASTLES[3]) { moves.push(CASTLES[3].clone()); }
            }
        }
    }

    fn get_pawn_moves(&mut self, coord: &Coord, moves: &mut Vec<Move>) {
        let (y, x) = coord.vals();
        let color = self.board[y][x].unwrap().color;
        let pawn_dir = if color == WHITE {-1} else {1};
        let will_promote = y == (if color == WHITE {1} else {6});

        if self.board[(y as isize + pawn_dir) as usize][x].is_none() {
            // Forward 1
            if will_promote {
                let promos = Move::get_promotions(*coord, Coord::new((y as isize + pawn_dir) as usize, x));
                if self.move_is_legal(&promos[0]) { moves.extend(promos); }
            } else {
                let mv = Move::new(*coord, Coord::new((y as isize + pawn_dir) as usize, x), MoveType::Basic);
                if self.move_is_legal(&mv) { moves.push(mv); }
            }
            // Forward 2
            if (color == WHITE && y == 6 || color == BLACK && y == 1) && self.board[(y as isize + 2*pawn_dir) as usize][x].is_none() {
                let mv = Move::new(*coord, Coord::new((y as isize + 2*pawn_dir) as usize, x), MoveType::Basic);
                if self.move_is_legal(&mv) { moves.push(mv); }
            }
        }

        if x != 0 {
            // Capture left
            if self.square_is_color(&Coord::new((y as isize + pawn_dir) as usize, x - 1), !color) {
                if will_promote {
                    let promos = Move::get_promotions(*coord, Coord::new((y as isize + pawn_dir) as usize, x - 1));
                    if self.move_is_legal(&promos[0]) { moves.extend(promos); }
                } else {
                    let mv = Move::new(*coord, Coord::new((y as isize + pawn_dir) as usize, x - 1), MoveType::Basic);
                    if self.move_is_legal(&mv) { moves.push(mv); }
                }
            }
            // En passant left
            if let Some(sq) = self.en_passant {
                if sq.y == (y as isize + pawn_dir) as usize && sq.x == x - 1 {
                    let mv = Move::new(*coord, Coord::new((y as isize + pawn_dir) as usize, x - 1), MoveType::EnPassant);
                    if self.move_is_legal(&mv) { moves.push(mv); }
                }
            }
        }
        if x != 7 {
            // Capture right
            if self.square_is_color(&Coord::new((y as isize + pawn_dir) as usize, x + 1), !color) {
                if will_promote {
                    let promos = Move::get_promotions(*coord, Coord::new((y as isize + pawn_dir) as usize, x + 1));
                    if self.move_is_legal(&promos[0]) { moves.extend(promos); }
                } else {
                    let mv = Move::new(*coord, Coord::new((y as isize + pawn_dir) as usize, x + 1), MoveType::Basic);
                    if self.move_is_legal(&mv) { moves.push(mv); }
                }
            }
            // En passant right
            if let Some(sq) = self.en_passant {
                if sq.y == (y as isize + pawn_dir) as usize && sq.x == x + 1 {
                    let mv = Move::new(*coord, Coord::new((y as isize + pawn_dir) as usize, x + 1), MoveType::EnPassant);
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

    fn king_is_attacked(&self, color: bool) -> bool {
        let king = Coord::ALL.iter().find(|c|
            self.square_is_color(*c, color) && self.square_is_piece_type(*c, PieceType::King)
        ).unwrap();

        self.square_is_attacked(king, !color)
    }

    fn square_is_attacked(&self, target: &Coord, color: bool) -> bool {
        self.find_players_pieces(color).any(|coord| self.piece_attacks(&coord, target))
    }

    fn piece_attacks(&self, coord: &Coord, target: &Coord) -> bool {
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
                let dir = if self.get_square(coord).unwrap().color == WHITE {-1} else {1};
                coord.x.abs_diff(target.x) == 1 && (coord.y as isize + dir) as usize == target.y
            },
        }
    }

    fn can_linearly_attack(&self, from: &Coord, to: &Coord, step_list: &[(isize, isize)]) -> bool {
        for step in step_list {
            let mut test_coord = *from;
            while test_coord.add(step) {
                if test_coord == *to {
                    return true;
                }
                if self.get_square(&test_coord).is_some() {
                    break;
                }
            }
        }
        return false;
    }

    fn update_state_no_moves(&mut self) {
        self.state = if self.is_check() {
            BoardState::Stalemate
        } else if self.side_to_move == WHITE {
            BoardState::WhiteWin
        } else {
            BoardState::BlackWin
        };
    }

    pub fn is_check(&self) -> bool {
        self.king_is_attacked(self.side_to_move)
    }

    // fn update_state_post_move(&mut self) {
    //     if self.check_threefold_repetition() {
    //         self.state = BoardState::ThreefoldRepetition;
    //     } else
    //     if self.halfmove_count >= 100 {
    //         self.state = BoardState::FiftyMoveRule;
    //     } else if self.check_insufficient_material() {
    //         self.state = BoardState::InsufficientMaterial;
    //     }
    // }

    // fn get_attacks(&self, color: bool) -> Vec<Move> {
    //     let mut attacks = Vec::new();
    //     for coord in self.find_players_pieces(color) {
    //         self.get_piece_moves(&coord, &mut attacks);
    //     }
    //     attacks
    // }

    // fn check_threefold_repetition(&self) -> bool {
    //     let Some(&curr_pos) = self.position_history.last() else { return false; };
    //     let mut idx = self.position_history.len() - 1;
    //     let mut count = 1;
    //     loop {
    //         if idx <= 2 {
    //             break;
    //         }
    //         idx -= 2;

    //         if self.position_history[idx] == curr_pos {
    //             count += 1;
    //             if count == 3 {
    //                 return true;
    //             }
    //         }
    //     }
    //     return false;
    // }

    // fn check_insufficient_material(&self) -> bool {
    //     return false; // do this
    // }
}