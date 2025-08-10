use crate::bchess::piece::{Piece, NUM_PIECES, PIECES};
use crate::bchess::square::{File, Rank, Square, FILES, RANKS};
use crate::bchess::color::{Color, NUM_COLORS, COLORS};
use crate::bchess::bitboard::Bitboard;

// use crate::chess::Move;
use crate::bchess::mv::{Move, MoveType};

pub const START_POS_FEN: &'static str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

#[derive(Debug, Clone, Copy)]
pub struct Castles {
    pub w_k: bool,
    pub w_q: bool,
    pub b_k: bool,
    pub b_q: bool
}

// pub struct Castles(u8);

// impl Castles {
//     pub const W_K: u8 = 8;
//     pub const W_Q: u8 = 4;
//     pub const B_K: u8 = 2;
//     pub const B_Q: u8 = 1;
//     pub const ALL: u8 = 15;

//     pub const fn new(castles: u8) -> Self {
//         Self(castles)
//     }

//     pub const fn is_set(&self, castle: u8) -> bool {
//         self.0 & castle != 0
//     }

//     pub const fn set(&mut self, castle: u8) {
//         self.0 |= castle;
//     }

//     pub const fn unset(&mut self, castle: u8) {
//         self.0 &= !castle
//     }
// }

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

struct MoveUndoer {
    mv: Move,
    captured: Option<(Piece, Color)>,
    en_passant: Option<Square>,
    castling: Castles,
    halfmoves: u32
}

pub struct Board {
    pieces: [Bitboard; 6],
    colors: [Bitboard; 2],
    side_to_move: Color,
    castling: Castles,
    en_passant: Option<Square>,
    halfmoves: u32,
    fullmoves: u32,
    state: BoardState,
    undoers: Vec<MoveUndoer>,
}

impl std::fmt::Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const fn write_piece(color: Color, piece: Piece) -> char {
            match (color, piece) {
                (Color::White, Piece::Rook) => 'R',
                (Color::White, Piece::Knight) => 'N',
                (Color::White, Piece::Bishop) => 'B',
                (Color::White, Piece::Queen) => 'Q',
                (Color::White, Piece::King) => 'K',
                (Color::White, Piece::Pawn) => 'P',
                (Color::Black, Piece::Rook) => 'r',
                (Color::Black, Piece::Knight) => 'n',
                (Color::Black, Piece::Bishop) => 'b',
                (Color::Black, Piece::Queen) => 'q',
                (Color::Black, Piece::King) => 'k',
                (Color::Black, Piece::Pawn) => 'p'
            }
        }

        let mut s = String::new();
        for rank in RANKS.into_iter().rev() {
            for file in FILES {
                let square = Square::from_coords(file, rank);
                if let Some(color) = self.get_color_at(square) {
                    let piece = self.get_piece_at(square).unwrap();
                    s.push(write_piece(color, piece));
                    s.push(' ');
                } else {
                    s += ". ";
                }
            }
            s.push('\n');
        }
        write!(f, "{}", s)
    }
}

impl Board {
    pub fn new(fen: &str) -> Option<Self> {
        let (pieces, colors, side_to_move, castling, en_passant, halfmoves, fullmoves) = Self::make_position(fen)?;
        Some(Self {
            pieces,
            colors,
            side_to_move,
            castling,
            en_passant,
            halfmoves,
            fullmoves,
            state: BoardState::Live,
            undoers: Vec::new()
        })
    }

    pub fn default() -> Self {
        Self::new(START_POS_FEN).unwrap()
        // unsafe { Self::new(START_POS_FEN).unwrap_unchecked() }
    }

    pub fn set_position(&mut self, fen: &str) -> bool {
        let Some((pieces, colors, side_to_move, castling, en_passant, halfmoves, fullmoves))
            = Self::make_position(fen) else { return false; };
        
        self.pieces = pieces;
        self.colors = colors;
        self.side_to_move = side_to_move;
        self.castling = castling;
        self.en_passant = en_passant;
        self.halfmoves = halfmoves;
        self.fullmoves = fullmoves;
        return true;
    }

    fn make_position(fen: &str) -> Option<(
        [Bitboard; 6], [Bitboard; 2], Color, Castles, Option<Square>, u32, u32
    )> {
        if fen.is_empty() || !fen.is_ascii() { return None; }

        let [board, side_to_move, castling, en_passant, halfmoves, fullmoves
        ] = fen.split(" ").collect::<Vec<_>>().try_into().ok()?;

        // Board
        let mut pieces = [Bitboard::EMPTY; NUM_PIECES];
        let mut colors = [Bitboard::EMPTY; NUM_COLORS];

        // TODO: check for repeated numbers (e.g. "44") in fen
        let mut rank = b'8';
        for row in board.split("/") {
            if rank < b'1' { return None; }

            let mut file = b'a';
            for char in row.bytes() {
                if file > b'h' { return None; }

                // Check if character is a number
                if char >= b'1' && char <= b'8' {
                    file += char - b'0';
                }
                else if let Some(piece) = Piece::from_ascii(char) {
                    let color = if char.is_ascii_uppercase() { Color::White } else { Color::Black };

                    let bb = Bitboard::from_square(Square::from_coords(File::from_ascii(file), Rank::from_ascii(rank)));
                    pieces[piece.idx()] ^= bb;
                    colors[color.idx()] ^= bb;
                    file += 1;
                }
                else {
                    return None;
                }
            }
            if file != b'i' { return None; }
            rank -= 1;
        }
        if rank != b'0' { return None; }

        // Side to move
        let side_to_move = match side_to_move {
            "w" => Color::White,
            "b" => Color::Black,
            _ => return None
        };

        // Castling avilability
        let castling = Castles {
            w_k: castling.contains("K"),
            w_q: castling.contains("Q"),
            b_k: castling.contains("k"),
            b_q: castling.contains("q"),
        };

        // En passant
        let en_passant = match en_passant {
            "-" => None,
            san => Some(Square::from_san(san)?)
        };

        // Halfmoves
        let Ok(halfmoves) = halfmoves.parse::<u32>() else { return None; };
        // Fullmoves
        let Ok(fullmoves) = fullmoves.parse::<u32>() else { return None; };

        Some((pieces, colors, side_to_move, castling, en_passant, halfmoves, fullmoves))
    }

    pub const fn get_side_to_move(&self) -> Color { self.side_to_move }
    pub const fn get_en_passant(&self) -> Option<Square> { self.en_passant }
    pub const fn get_state(&self) -> BoardState { self.state }

    pub const fn is_live(&self) -> bool {
        match self.state {
            BoardState::Live => true,
            _ => false
        }
    }

    pub fn is_piece_at(&self, square: Square) -> bool {
        let square = Bitboard::from_square(square);
        for &bb in &self.colors {
            if bb & square != Bitboard::EMPTY { return true; }
        }
        false
    }

    pub fn get_piece_at(&self, square: Square) -> Option<Piece> {
        let square = Bitboard::from_square(square);
        for piece in PIECES {
            if self.pieces[piece.idx()] & square != Bitboard::EMPTY {
                return Some(piece);
            }
        }
        None
    }

    pub fn get_color_at(&self, square: Square) -> Option<Color> {
        let bb = Bitboard::from_square(square);
        for color in COLORS {
            if self.colors[color.idx()] & bb != Bitboard::EMPTY {
                return Some(color);
            }
        }
        None
    }

    pub fn xor(&mut self, piece: Piece, color: Color, bitboard: Bitboard) {
        self.pieces[piece.idx()] ^= bitboard;
        self.colors[color.idx()] ^= bitboard;
    } 

    pub fn make_move(&mut self, mv: &Move, undoable: bool) {
        if !self.is_live() { return; }
        // Only legal moves should make it to this function
        let from = mv.get_from();
        let to = mv.get_to();
        let move_type = mv.get_move_type();

        let from_bb = Bitboard::from_square(from);
        let to_bb = Bitboard::from_square(to);

        let piece = self.get_piece_at(from).unwrap();
        let my_color = self.get_color_at(from).unwrap();

        let is_capture = self.is_piece_at(to);
        let captured = self.get_piece_at(to).zip(self.get_color_at(to));

        // Make a move undoer if undoable, or clear the undo stack if not
        if undoable {
            self.undoers.push(MoveUndoer {
                mv: mv.clone(),
                captured,
                en_passant: self.en_passant,
                castling: self.castling,
                halfmoves: self.halfmoves
            });
        } else {
            self.undoers.clear();
        }

        // Make the swap
        let end_piece = if let MoveType::Promotion(to) = mv.get_move_type() {
            to
        } else {
            piece
        };

        self.xor(piece, my_color, from_bb);
        self.xor(end_piece, my_color, to_bb);
        if let Some((captured_piece, _)) = captured {
            self.xor(captured_piece, !my_color, to_bb);
        }

        // En passant
        const fn ep_square(from: Square, to: Square) -> Square {
            Square::from_coords(to.file(), from.rank())
        }

        if mv.get_move_type() == MoveType::EnPassant {
            self.xor(Piece::Pawn, !my_color, Bitboard::from_square(ep_square(from, to)));
        }

        // Castling
        if mv.get_move_type() == MoveType::Castle {
            let (from_file, to_file) = match to.file() {
                File::C => (File::A, File::D),
                File::G => (File::H, File::F),
                _ => unreachable!()
            };
            let rank = match my_color {
                Color::White => Rank::One,
                Color::Black => Rank::Eight
            };
            self.xor(Piece::Rook, my_color, Bitboard::from_square(Square::from_coords(from_file, rank)));
            self.xor(Piece::Rook, my_color, Bitboard::from_square(Square::from_coords(to_file, rank)));
        }

        match (from.file(), from.rank()) {
            (File::E, Rank::One) => {
                self.castling.w_k = false;
                self.castling.w_q = false;
            },
            (File::E, Rank::Eight) => {
                self.castling.b_k = false;
                self.castling.b_q = false;
            },
            (File::H, Rank::One) => { self.castling.w_k = false; },
            (File::A, Rank::One) => { self.castling.w_q = false; },
            (File::H, Rank::Eight) => { self.castling.b_k = false; },
            (File::A, Rank::Eight) => { self.castling.b_q = false; },
            _ => ()
        };

        // Update en passant square
        if mv.get_move_type() == MoveType::FirstPawnMove {
            self.en_passant = Some(to.backward(my_color).unwrap());
        } else {
            self.en_passant = None;
        }

        // Update halfmove count
        if piece == Piece::Pawn || captured.is_some() || mv.get_move_type() == MoveType::EnPassant {
            self.halfmoves = 0;
        } else {
            self.halfmoves += 1;
        }

        // Update fullmove count after black moves
        if self.side_to_move.is_black() { self.fullmoves += 1; }

        // Update turn
        self.side_to_move = !self.side_to_move;

        // TODO: update state
        // TODO: add hash to history
    }

    pub fn undo_move(&mut self) {
        todo!()
    }

    pub fn gen_legal_moves(&mut self, v: &mut Vec<Move>) {
        todo!()
    }
}