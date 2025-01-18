use crate::{Move, MoveType, Piece, PieceType};
use crate::coord::{Coord, is_on_board};

struct UndoData {
    captured: Option<Piece>,
    en_passant: Option<Coord>,
    allowed_castling: (bool, bool, bool, bool),
    halfmove_count: usize,
}

pub struct Board {
    board: [[Option<Piece>; 8]; 8],
    side_to_move: bool,
    allowed_castling: (bool, bool, bool, bool), // KQkq
    en_passant: Option<Coord>,
    halfmove_count: usize,
    fullmove_num: usize,
    undo_stack: Vec<UndoData>
}

const WHITE: bool = true;
const BLACK: bool = false;

const R_STEPS: [(isize, isize); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];
const N_STEPS: [(isize, isize); 8] = [(2, 1), (2, -1), (1, 2), (1, -2), (-1, 2), (-1, -2), (-2, 1), (-2, -1)];
const B_STEPS: [(isize, isize); 4] = [(1, 1), (1, -1), (-1, 1), (-1, -1)];
const KQ_STEPS: [(isize, isize); 8] = [(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (1, -1), (-1, 1), (-1, -1)];

const CASTLES: [Move; 4] = [
    Move {
        from: Coord::from(7, 4),
        to: Coord::from(7, 6),
        move_type: MoveType::Castle
    },
    Move {
        from: Coord::from(7, 4),
        to: Coord::from(7, 2),
        move_type: MoveType::Castle
    },
    Move {
        from: Coord::from(0, 4),
        to: Coord::from(0, 6),
        move_type: MoveType::Castle
    },
    Move {
        from: Coord::from(0, 4),
        to: Coord::from(0, 2),
        move_type: MoveType::Castle
    }
];

impl std::fmt::Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut board_str = String::from("\n");
        for row in self.board {
            for cell in row {
                board_str += (match cell {
                    Some(p) => p.to_string(),
                    None => String::from(".")
                } + " ").as_str();
            }
            board_str += "\n";
        }
        write!(f, "{}", board_str)
    }
}

impl Board {
    pub fn from_fen(fen: &str) -> Option<Self> {
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

                if let Some(piece) = Piece::from_char(p) {
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
        let Ok(halfmove_count) = halfmove_count.parse::<usize>() else { return None; };

        let Some(fullmove_num) = fen_fields.next() else { return None; };
        let Ok(fullmove_num) = fullmove_num.parse::<usize>() else { return None; };

        if fen_fields.count() == 0 {
            Some(Board {
                board,
                side_to_move,
                allowed_castling,
                en_passant,
                halfmove_count,
                fullmove_num,
                undo_stack: Vec::new()
            })
        } else {
            None
        }
    }

    pub fn default() -> Self {
        Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap()
    }

    pub fn fen(&self) -> String {
        let board = (0..8).into_iter().map(|y| {
            let mut row = String::new();
            let mut gap: u8 = 0;
            for x in 0..8 {
                match self.board[y][x] {
                    Some(p) => {
                        if gap > 0 {
                            row += gap.to_string().as_str();
                            gap = 0;
                        }
                        row += p.to_string().as_str();
                    },
                    None => gap += 1
                }
            }
            if gap > 0 {
                row += gap.to_string().as_str();
            }
            row
        }).collect::<Vec<String>>().join("/");

        let side_to_move = if self.side_to_move {"w"} else {"b"};

        let mut castling = String::with_capacity(4);
        if self.allowed_castling.0 { castling.push('K'); }
        if self.allowed_castling.1 { castling.push('Q'); }
        if self.allowed_castling.2 { castling.push('k'); }
        if self.allowed_castling.3 { castling.push('q'); }
        if castling == "" { castling.push('-'); }

        let en_passant = match self.en_passant {
            Some(c) => c.to_string(),
            None => "-".to_string()
        };

        format!("{} {} {} {} {} {}",
            board,
            side_to_move,
            castling,
            en_passant,
            self.halfmove_count,
            self.fullmove_num
        )
    }

    pub fn make_move(&mut self, mv: &Move) {
        // Only legal moves should make it to this function
        let (from_y, from_x) = mv.from.tup();
        let (to_y, to_x) = mv.to.tup();
        let piece = self.board[from_y][from_x].unwrap();

        let (captured, is_capture) = match self.board[to_y][to_x] {
            Some(p) => (Some(p), true),
            None => (None, mv.move_type == MoveType::EnPassant)
        };

        // Add data to undo this move
        self.undo_stack.push(UndoData {
            captured,
            en_passant: self.en_passant,
            allowed_castling: self.allowed_castling,
            halfmove_count: self.halfmove_count
        });

        // Update halfmove count
        if piece.piece_type == PieceType::Pawn || is_capture {
            self.halfmove_count = 0;
        } else {
            self.halfmove_count += 1;
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
            let (f_x, t_x) = match to_x {
                6 => (7, 5),
                2 => (0, 3),
                _ => panic!("Error: invalid castle")
            };
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
        if piece.piece_type == PieceType::Pawn && (if to_y > from_y {to_y - from_y == 2} else {from_y - to_y == 2}) {
            self.en_passant = Some(Coord::from((to_y as isize - {if piece.color {-1} else {1}}) as usize, to_x));
        } else {
            self.en_passant = None;
        }

        // Update fullmove num after black moves
        if !self.side_to_move {self.fullmove_num += 1;}
        // Update turn
        self.side_to_move = !self.side_to_move;
    }

    // fn make_moves(&mut self, moves: Vec<&Move>) {
    //     for mv in moves {
    //         self.make_move(mv);
    //     }
    // }

    pub fn undo_move(&mut self, mv: &Move) {
        let Some(undo_data) = self.undo_stack.pop() else {return};

        let (from_y, from_x) = mv.from.tup();
        let (to_y, to_x) = mv.to.tup();
        let piece = self.board[to_y][to_x].unwrap();

        // Swap
        self.board[from_y][from_x] = if let MoveType::Promotion(_) = mv.move_type {
            Some(Piece {
                piece_type: PieceType::Pawn,
                color: piece.color
            })
        } else {
            Some(piece)
        };
        self.board[to_y][to_x] = undo_data.captured;

        if mv.move_type == MoveType::EnPassant {
            self.board[from_y][to_x] = Some(Piece {
                piece_type: PieceType::Pawn,
                color: self.side_to_move
            });
        }

        if mv.move_type == MoveType::Castle {
            let (f_x, t_x) = match to_x {
                6 => (7, 5),
                2 => (0, 3),
                _ => panic!("Error: invalid castle")
            };
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

    // fn undo_moves(&mut self, moves: Vec<&Move>) {
    //     for mv in moves {
    //         self.undo_move(mv);
    //     }
    // }

    pub fn get_square(&self, coord: Coord) -> Option<Piece> {
        let (y, x) = coord.tup();
        self.board[y][x]
    }

    pub fn get_side_to_move(&self) -> bool {
        self.side_to_move
    }

    pub fn square_is_color(&self, coord: Coord, color: bool) -> bool {
        let (y, x) = coord.tup();
        match self.board[y][x] {
            Some(piece) => piece.color == color,
            None => false
        }
    }

    pub fn square_is_piece_type(&self, coord: Coord, piece_type: PieceType) -> bool {
        let (y, x) = coord.tup();
        match self.board[y][x] {
            Some(piece) => piece.piece_type == piece_type,
            None => false
        }
    }

    pub fn square_is_piece(&self, coord: Coord, color: bool, piece_type: PieceType) -> bool {
        self.square_is_color(coord, color) && self.square_is_piece_type(coord, piece_type)
    }

    pub fn find_players_pieces<'a>(&'a self, color: bool) -> impl Iterator<Item = Coord> + 'a {
        Coord::all()
        .filter(move |&c| self.square_is_color(c, color))
    }

    fn get_linear_moves(&self, y: usize, x: usize, step_list: &[(isize, isize)], one_step_only: bool) -> Vec<Move> {
        let color = self.board[y][x].unwrap().color;
        let mut moves = Vec::new();
        for (step_y, step_x) in step_list {
            let mut test_y = (y as isize + step_y) as usize;
            let mut test_x = (x as isize + step_x) as usize;
            while is_on_board(test_y, test_x) {
                if !self.square_is_color(test_y, test_x, color) {
                    moves.push(Move::new(Coord::from(y, x), Coord::from(test_y, test_x), MoveType::Basic));
                    if self.square_is_color(test_y, test_x, !color) {
                        break;
                    }
                } else {
                    break;
                }
                
                test_y = (test_y as isize + step_y) as usize;
                test_x = (test_x as isize + step_x) as usize;

                if one_step_only {
                    break;
                }
            }
        }
        moves
    }

    fn get_rook_moves(&self, y: usize, x: usize) -> Vec<Move> {
        self.get_linear_moves(y, x, &R_STEPS, false)
    }
    fn get_knight_moves(&self, y: usize, x: usize) -> Vec<Move> {
        self.get_linear_moves(y, x, &N_STEPS, true)
    }
    fn get_bishop_moves(&self, y: usize, x: usize) -> Vec<Move> {
        self.get_linear_moves(y, x, &B_STEPS, false)
    }
    fn get_queen_moves(&self, y: usize, x: usize) -> Vec<Move> {
        self.get_linear_moves(y, x, &KQ_STEPS, false)
    }
    fn castling_is_ok(&self, castle: usize, y: usize, x: usize) -> bool {
        let (req_y, allowed, empty) = match castle {
            0 => (7, self.allowed_castling.0, [(7, 5), (7, 5), (7, 6)]), // duplicate items to line up sizes :skull:
            1 => (7, self.allowed_castling.1, [(7, 1), (7, 2), (7, 3)]),
            2 => (0, self.allowed_castling.2, [(0, 5), (0, 5), (0, 6)]),
            3 => (0, self.allowed_castling.3, [(0, 1), (0, 2), (0, 3)]),
            x => panic!("castling_is_ok: illegal `castle` arg: {}", x)
        };
        y == req_y && x == 4 && allowed && empty.into_iter().all(|(y, x)| self.board[y][x].is_none())
    }

    fn get_king_moves(&self, y: usize, x: usize) -> Vec<Move> {
        let mut moves: Vec<Move> = self.get_linear_moves(y, x, &KQ_STEPS, true);

        for castle in 0..4 {
            if self.castling_is_ok(castle, y, x) {
                moves.push(CASTLES[castle].clone());
            }
        }
        moves
    }

    fn get_pawn_moves(&self, y: usize, x: usize) -> Vec<Move> {
        let color = self.board[y][x].unwrap().color;
        let pawn_dir = if color {-1} else {1};
        let will_promote = (y as isize + pawn_dir) == {if color {0} else {7}};
        let mut moves = Vec::new();
        if self.board[(y as isize + pawn_dir) as usize][x].is_none() {
            if will_promote {
                // Promotion moves
                moves.extend(Move::promotions(Coord::from(y, x), Coord::from((y as isize + pawn_dir) as usize, x)));
            } else {
                // Basic move
                moves.push(Move::new(Coord::from(y, x), Coord::from((y as isize + pawn_dir) as usize, x), MoveType::Basic));
            }
            // Starting move
            if (color && y == 6) || (!color && y == 1) {
                if self.board[(y as isize + 2*pawn_dir) as usize][x].is_none() {
                    moves.push(Move::new(Coord::from(y, x), Coord::from((y as isize + 2*pawn_dir) as usize, x), MoveType::Basic));
                }
            }
        }

        if x != 0 {
            // Capture left
            if self.square_is_color((y as isize + pawn_dir) as usize, x - 1, !color) {
                if will_promote {
                    // Capture left and promote
                    moves.extend(Move::promotions(Coord::from(y, x), Coord::from((y as isize + pawn_dir) as usize, x - 1)));
                } else {
                    // Don't promote
                    moves.push(Move::new(Coord::from(y, x), Coord::from((y as isize + pawn_dir) as usize, x - 1), MoveType::Basic));
                }
                // En passant left
                if let Some(sq) = self.en_passant {
                    if sq.tup() == ((y as isize + pawn_dir) as usize, x - 1) {
                        moves.push(Move::new(Coord::from(y, x), Coord::from((y as isize + pawn_dir) as usize, x - 1), MoveType::EnPassant));
                    }
                }
            }
        }
        if x != 7 {
            // Capture right
            if self.square_is_color((y as isize + pawn_dir) as usize, x + 1, !color) {
                if will_promote {
                    // Capture right and promote
                    moves.extend(Move::promotions(Coord::from(y, x), Coord::from((y as isize + pawn_dir) as usize, x + 1)));
                } else {
                    // Don't promote
                    moves.push(Move::new(Coord::from(y, x), Coord::from((y as isize + pawn_dir) as usize, x + 1), MoveType::Basic));
                }
            }
            // En passant right
            if let Some(sq) = self.en_passant {
                if sq.tup() == ((y as isize + pawn_dir) as usize, x + 1) {
                    moves.push(Move::new(Coord::from(y, x), Coord::from((y as isize + pawn_dir) as usize, x + 1), MoveType::EnPassant));
                }
            }
        }
        moves
    }

    fn get_piece_moves(&self, y: usize, x: usize) -> Vec<Move> {
        let piece = self.board[y][x].unwrap();
        match piece.piece_type {
            PieceType::Rook => self.get_rook_moves(y, x),
            PieceType::Knight => self.get_knight_moves(y, x),
            PieceType::Bishop => self.get_bishop_moves(y, x),
            PieceType::Queen => self.get_queen_moves(y, x),
            PieceType::King => self.get_king_moves(y, x),
            PieceType::Pawn => self.get_pawn_moves(y, x),
        }
    }

    fn get_attacks<'a>(&'a self, color: bool) -> impl Iterator<Item = Move> + 'a {
        self.find_players_pieces(color)
        .flat_map(|c| self.get_piece_moves(c.tup().0, c.tup().1)) // make nicer?
    }

    fn king_is_attacked(&self, color: bool) -> bool {
        let king = Coord::all_tup()
        .find(|&(y, x)|
            match self.board[y][x] {
                Some(piece) => piece.piece_type == PieceType::King && piece.color == color,
                None => false
            }
        ).unwrap();

        self.get_attacks(!color)
            .any(|mv| mv.to.tup() == king)
    }

    pub fn get_legal_moves<'a>(&mut self) -> Vec<Move> {
        self.get_attacks(self.side_to_move).collect::<Vec<Move>>().into_iter()
        .filter(|mv| {
            self.make_move(mv);
            let is_legal = !self.king_is_attacked(!self.side_to_move);
            self.undo_move(mv);
            is_legal
        }).collect()
    }

    pub fn is_check(&self) -> bool {
        self.king_is_attacked(self.side_to_move)
    }

    pub fn fifty_move_rule(&self) -> bool {
        self.halfmove_count >= 100
    }
}