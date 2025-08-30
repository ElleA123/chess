use crate::bchess::magic_tables;
use crate::bchess::mv::{Move, MoveType};

use super::bitboard::Bitboard;
use super::square::*;
use super::color::*;
use super::piece::*;

pub const START_POS_FEN: &'static str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

#[derive(Debug, Clone, Copy)]
pub enum Castle {
    WK = 1,
    WQ = 2,
    BK = 4,
    BQ = 8
}

const NUM_CASTLES: usize = 4;

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct Castles(u8);

impl Castles {
    pub const ALL: Self = Self(15);
    pub const NONE: Self = Self(0);

    pub const fn new(castles: u8) -> Self {
        Self(castles)
    }

    pub const fn is_set(&self, castle: Castle) -> bool {
        self.0 & castle as u8 != 0
    }

    pub const fn set(&mut self, castle: Castle) {
        self.0 |= castle as u8;
    }

    pub const fn unset(&mut self, castle: Castle) {
        self.0 &= !(castle as u8);
    }
}

pub const CASTLE_WK_MOVE: Move = Move {
    from: Square::E1,
    to: Square::G1,
    move_type: MoveType::Castle
};
pub const CASTLE_WQ_MOVE: Move = Move {
    from: Square::E1,
    to: Square::C1,
    move_type: MoveType::Castle
};
pub const CASTLE_BK_MOVE: Move = Move {
    from: Square::E8,
    to: Square::G8,
    move_type: MoveType::Castle
};
pub const CASTLE_BQ_MOVE: Move = Move {
    from: Square::E8,
    to: Square::C8,
    move_type: MoveType::Castle
};

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

#[derive(Clone, Copy)]
pub struct Board {
    pieces: [Bitboard; 6],
    colors: [Bitboard; 2],
    side_to_move: Color,
    castles: Castles,
    en_passant: Option<Square>,
    halfmoves: u8,
}

impl Board {
    pub fn new(fen: &str) -> Option<Self> {
        if !fen.is_ascii() || fen.is_empty() { return None; }

        let [
            board, side_to_move, allowed_castling, en_passant, halfmove_count, fullmove_num
        ] = fen.trim().split(" ").collect::<Vec<_>>().try_into().ok()?;

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

        // Castling avilability - TODO: add error handling
        let mut castles = Castles::NONE;
        if allowed_castling.contains("K") { castles.set(Castle::WK); }
        if allowed_castling.contains("Q") { castles.set(Castle::WQ); }
        if allowed_castling.contains("k") { castles.set(Castle::BK); }
        if allowed_castling.contains("q") { castles.set(Castle::BQ); }

        // En passant
        let en_passant = match en_passant {
            "-" => None,
            san => Some(Square::from_san(san)?)
        };

        // Halfmove count
        let Ok(halfmoves) = halfmove_count.parse::<u8>() else { return None; };
        // Fullmove num
        let Ok(_) = fullmove_num.parse::<u32>() else { return None; };

        Some(Self { pieces, colors, side_to_move, castles, en_passant, halfmoves })
    }

    #[inline]
    pub fn default() -> Self {
        Self::new(START_POS_FEN).unwrap()
    }

    #[inline]
    pub const fn get_piece(&self, piece: Piece) -> Bitboard {
        self.pieces[piece.idx()]
    }

    #[inline]
    pub const fn get_color(&self, color: Color) -> Bitboard {
        self.colors[color.idx()]
    }

    #[inline]
    pub const fn get_side_to_move(&self) -> Color {
        self.side_to_move
    }

    #[inline]
    pub fn get_piece_at(&self, square: Square) -> Option<Piece> {
        let square = Bitboard::from_square(square);

        if (self.pieces[Piece::Rook.idx()] | self.pieces[Piece::Knight.idx()] | self.pieces[Piece::Bishop.idx()]) & square != Bitboard::EMPTY {
            if self.pieces[Piece::Rook.idx()] & square != Bitboard::EMPTY {
                return Some(Piece::Rook);
            }
            if self.pieces[Piece::Knight.idx()] & square != Bitboard::EMPTY {
                return Some(Piece::Knight);
            }
            return Some(Piece::Bishop);
        }
        else if (self.pieces[Piece::Queen.idx()] | self.pieces[Piece::King.idx()] | self.pieces[Piece::Pawn.idx()]) & square != Bitboard::EMPTY {
            if self.pieces[Piece::Queen.idx()] & square != Bitboard::EMPTY {
                return Some(Piece::Queen);
            }
            if self.pieces[Piece::King.idx()] & square != Bitboard::EMPTY {
                return Some(Piece::King);
            }
            return Some(Piece::Pawn);
        }
        return None;
        // for (piece, bitboard) in PIECES.into_iter().zip(&self.pieces) {
        //     if *bitboard & square != Bitboard::EMPTY {
        //         return Some(piece);
        //     }
        // }
        // None
    }

    #[inline]
    pub fn get_color_at(&self, square: Square) -> Option<Color> {
        let square = Bitboard::from_square(square);
        for color in COLORS {
            if self.colors[color.idx()] & square != Bitboard::EMPTY {
                return Some(color);
            }
        }
        None
    }

    #[inline(always)]
    pub const fn get_en_passant(&self) -> Option<Square> { self.en_passant }

    #[inline(always)]
    pub fn blockers(&self) -> Bitboard {
        self.colors[Color::White.idx()] | self.colors[Color::Black.idx()]
    }

    #[inline]
    pub fn is_check(&self) -> bool {
        self.pieces[Piece::King.idx()] & self.colors[(!self.side_to_move).idx()]
        & gen_attacks(self, self.side_to_move, self.blockers()) != Bitboard::EMPTY
    }
}

impl std::fmt::Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const fn write_piece(color: Color, piece: Piece) -> char {
            match color {
                Color::White => match piece {
                    Piece::Rook => 'R',
                    Piece::Knight => 'N',
                    Piece::Bishop => 'B',
                    Piece::Queen => 'Q',
                    Piece::King => 'K',
                    Piece::Pawn => 'P'
                },
                Color::Black => match piece {
                    Piece::Rook => 'r',
                    Piece::Knight => 'n',
                    Piece::Bishop => 'b',
                    Piece::Queen => 'q',
                    Piece::King => 'k',
                    Piece::Pawn => 'p'
                },
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

impl std::fmt::Debug for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "rooks:{}\nknights:{}\nbishops:{}\nqueens:{}\nkings:{}\npawns:{}\nwhite:{}\nblack:{}\nside_to_move:{:?}\ncastles:{}{}{}{}\nen_passant:{:?}\nhalfmoves:{}",
        self.pieces[Piece::Rook.idx()], self.pieces[Piece::Knight.idx()], self.pieces[Piece::Bishop.idx()], self.pieces[Piece::Queen.idx()], self.pieces[Piece::King.idx()], self.pieces[Piece::Pawn.idx()],
        self.colors[Color::White.idx()], self.colors[Color::Black.idx()],
        self.side_to_move,
        if self.castles.is_set(Castle::WK) {"K"} else {""},
        if self.castles.is_set(Castle::WQ) {"Q"} else {""},
        if self.castles.is_set(Castle::BK) {"k"} else {""},
        if self.castles.is_set(Castle::BQ) {"q"} else {""},
        self.en_passant, self.halfmoves)
    }
}

pub fn make_move(board: &Board, mv: Move) -> Board {
    #[inline(always)]
    fn xor(pieces: &mut [Bitboard; 6], colors: &mut [Bitboard; 2], bitboard: Bitboard, piece: Piece, color: Color) {
        pieces[piece.idx()] ^= bitboard;
        colors[color.idx()] ^= bitboard;
    }

    // Only legal moves should make it to this function
    let from_bb = Bitboard::from_square(mv.from);
    let to_bb = Bitboard::from_square(mv.to);

    let piece = board.get_piece_at(mv.from).unwrap();
    let captured = board.get_piece_at(mv.to);

    // Make the swap
    let mut pieces = board.pieces;
    let mut colors = board.colors;

    let end_piece = match mv.move_type {
        MoveType::Promotion(to) => to,
        _ => piece
    };

    xor(&mut pieces, &mut colors, from_bb, piece, board.side_to_move);
    xor(&mut pieces, &mut colors, to_bb, end_piece, board.side_to_move);
    if let Some(captured) = captured {
        xor(&mut pieces, &mut colors, to_bb, captured, !board.side_to_move);
    }

    // Castling move
    if mv.move_type == MoveType::Castle {
        let [from_file, to_file] = match mv.to.file() {
            File::C => [File::A, File::D],
            File::G => [File::H, File::F],
            _ => unreachable!()
        };
        let rank = match board.side_to_move {
            Color::White => Rank::One,
            Color::Black => Rank::Eight
        };
        xor(&mut pieces, &mut colors, Bitboard::from_square(Square::from_coords(from_file, rank)), Piece::Rook, board.side_to_move);
        xor(&mut pieces, &mut colors, Bitboard::from_square(Square::from_coords(to_file, rank)), Piece::Rook, board.side_to_move);
    }

    // En passant capture
    if mv.move_type == MoveType::EnPassant {
        xor(&mut pieces, &mut colors, Bitboard::from_square(
            Square::from_coords(mv.to.file(), mv.from.rank())
        ), Piece::Pawn, !board.side_to_move);
    }

    // Update turn
    let side_to_move = !board.side_to_move;

    // Update castles
    const CASTLE_POINTS: Bitboard = Bitboard(
        Bitboard::from_square(Square::A1).0 | Bitboard::from_square(Square::E1).0 | Bitboard::from_square(Square::H1).0 |
        Bitboard::from_square(Square::A8).0 | Bitboard::from_square(Square::E8).0 | Bitboard::from_square(Square::H8).0
    );

    let mut castles = board.castles;

    let move_bb = from_bb | to_bb;
    if move_bb & CASTLE_POINTS != Bitboard::EMPTY {
        if move_bb & Bitboard::from_square(Square::E1) != Bitboard::EMPTY {
            castles.unset(Castle::WK);
            castles.unset(Castle::WQ);
        } else if move_bb & Bitboard::from_square(Square::E8) != Bitboard::EMPTY {
            castles.unset(Castle::BK);
            castles.unset(Castle::BQ);
        }
        else {
            if move_bb & Bitboard::from_square(Square::H1) != Bitboard::EMPTY {
                castles.unset(Castle::WK);
            }
            if move_bb & Bitboard::from_square(Square::A1) != Bitboard::EMPTY {
                castles.unset(Castle::WQ);
            }
            if move_bb & Bitboard::from_square(Square::H8) != Bitboard::EMPTY {
                castles.unset(Castle::BK);
            }
            if move_bb & Bitboard::from_square(Square::A8) != Bitboard::EMPTY {
                castles.unset(Castle::BQ);
            }
        }
    }

    // Update en passant square
    let en_passant = match mv.move_type {
        MoveType::FirstPawnMove => Some(mv.to.backward(board.side_to_move).unwrap()),
        _ => None
    };

    // Update halfmove count
    let halfmoves = if piece == Piece::Pawn || captured.is_some() || mv.move_type == MoveType::EnPassant {
        0
    } else {
        board.halfmoves + 1
    };

    Board {
        pieces,
        colors,
        side_to_move,
        castles,
        en_passant,
        halfmoves
    }
}

pub fn gen_legal_moves(board: &Board, v: &mut Vec<Move>) {
    let mut pseudolegals = Vec::new();
    let blockers = board.blockers();

    for piece in PIECES {
        for square in board.pieces[piece.idx()] & board.colors[board.side_to_move.idx()] {
            gen_piece_moves(board, piece, square, blockers, &mut pseudolegals);
        }
    }

    // Legality check
    v.extend(pseudolegals.into_iter()
        .filter(|&mv| {
            let board = make_move(board, mv);
            board.pieces[Piece::King.idx()] & board.colors[(!board.side_to_move).idx()]
            & gen_attacks(&board, board.side_to_move, board.blockers()) == Bitboard::EMPTY
        })
    );
}

fn gen_piece_moves(board: &Board, piece: Piece, square: Square, blockers: Bitboard, v: &mut Vec<Move>) {
    match piece {
        Piece::Rook => {
            v.extend(magic_tables::get_rook_moves(square, blockers)
                .filter(|&to| board.colors[board.side_to_move.idx()] & Bitboard::from_square(to) == Bitboard::EMPTY)
                .map(|to| Move { from: square, to, move_type: MoveType::Basic })
            );
        },
        Piece::Knight => {
            v.extend(KNIGHT_MOVES[square.idx()]
                .filter(|&to| board.colors[board.side_to_move.idx()] & Bitboard::from_square(to) == Bitboard::EMPTY)
                .map(|to| Move { from: square, to, move_type: MoveType::Basic })
            );
        },
        Piece::Bishop => {
            v.extend(magic_tables::get_bishop_moves(square, blockers)
                .filter(|&to| board.colors[board.side_to_move.idx()] & Bitboard::from_square(to) == Bitboard::EMPTY)
                .map(|to| Move { from: square, to, move_type: MoveType::Basic })
            );
        },
        Piece::Queen => {
            v.extend(magic_tables::get_queen_moves(square, blockers)
                .filter(|&to| board.colors[board.side_to_move.idx()] & Bitboard::from_square(to) == Bitboard::EMPTY)
                .map(|to| Move { from: square, to, move_type: MoveType::Basic })
            );
        },
        Piece::King => {
            v.extend(KING_MOVES[square.idx()]
                .filter(|&to| board.colors[board.side_to_move.idx()] & Bitboard::from_square(to) == Bitboard::EMPTY)
                .map(|to| Move { from: square, to, move_type: MoveType::Basic })
            );

            const CASTLE_WK_EMPTY: Bitboard = Bitboard(Bitboard::from_square(Square::F1).0 | Bitboard::from_square(Square::G1).0);
            const CASTLE_WQ_EMPTY: Bitboard = Bitboard(Bitboard::from_square(Square::B1).0 | Bitboard::from_square(Square::C1).0 | Bitboard::from_square(Square::D1).0);
            const CASTLE_BK_EMPTY: Bitboard = Bitboard(Bitboard::from_square(Square::F8).0 | Bitboard::from_square(Square::G8).0);
            const CASTLE_BQ_EMPTY: Bitboard = Bitboard(Bitboard::from_square(Square::B8).0 | Bitboard::from_square(Square::C8).0 | Bitboard::from_square(Square::D8).0);

            const CASTLE_WK_UNATTACKED: Bitboard = Bitboard(Bitboard::from_square(Square::E1).0 | Bitboard::from_square(Square::F1).0 | Bitboard::from_square(Square::G1).0);
            const CASTLE_WQ_UNATTACKED: Bitboard = Bitboard(Bitboard::from_square(Square::C1).0 | Bitboard::from_square(Square::D1).0 | Bitboard::from_square(Square::E1).0);
            const CASTLE_BK_UNATTACKED: Bitboard = Bitboard(Bitboard::from_square(Square::E8).0 | Bitboard::from_square(Square::F8).0 | Bitboard::from_square(Square::G8).0);
            const CASTLE_BQ_UNATTACKED: Bitboard = Bitboard(Bitboard::from_square(Square::C8).0 | Bitboard::from_square(Square::D8).0 | Bitboard::from_square(Square::E8).0);

            let attacks = gen_attacks(board, !board.side_to_move, blockers);

            match board.side_to_move {
                Color::White => {
                    if board.castles.is_set(Castle::WK)
                    && blockers & CASTLE_WK_EMPTY == Bitboard::EMPTY
                    && attacks & CASTLE_WK_UNATTACKED == Bitboard::EMPTY {
                        v.push(CASTLE_WK_MOVE);
                    }
                    if board.castles.is_set(Castle::WQ)
                    && blockers & CASTLE_WQ_EMPTY == Bitboard::EMPTY
                    && attacks & CASTLE_WQ_UNATTACKED == Bitboard::EMPTY {
                        v.push(CASTLE_WQ_MOVE);
                    }
                },
                Color::Black => {
                    if board.castles.is_set(Castle::BK)
                    && blockers & CASTLE_BK_EMPTY == Bitboard::EMPTY
                    && attacks & CASTLE_BK_UNATTACKED == Bitboard::EMPTY {
                        v.push(CASTLE_BK_MOVE);
                    }
                    if board.castles.is_set(Castle::BQ)
                    && blockers & CASTLE_BQ_EMPTY == Bitboard::EMPTY
                    && attacks & CASTLE_BQ_UNATTACKED == Bitboard::EMPTY {
                        v.push(CASTLE_BQ_MOVE);
                    }
                }
            }
        },
        Piece::Pawn => {
            let mut pawn_moves = Vec::new();
            // Forward 1
            let fwd = square.forward(board.side_to_move).unwrap();
            if blockers & Bitboard::from_square(fwd) == Bitboard::EMPTY {
                pawn_moves.push(Move { from: square, to: fwd, move_type: MoveType::Basic });

                // Forward 2
                if square.rank() == match board.side_to_move {
                    Color::White => Rank::Two,
                    Color::Black => Rank::Seven
                } {
                    let fwd_2 = square.forward(board.side_to_move).unwrap()
                                            .forward(board.side_to_move).unwrap();
                    if blockers & Bitboard::from_square(fwd_2) == Bitboard::EMPTY {
                        pawn_moves.push(Move { from: square, to: fwd_2, move_type: MoveType::FirstPawnMove });
                    }
                }
            }

            // Capture left
            if let Some(capture) = PAWN_LEFT_CAPTURES[board.side_to_move.idx()][square.idx()] {
                if board.colors[(!board.side_to_move).idx()] & Bitboard::from_square(capture) != Bitboard::EMPTY {
                    pawn_moves.push(Move { from: square, to: capture, move_type: MoveType::Basic });
                }
                else if board.en_passant == Some(capture) {
                    pawn_moves.push(Move { from: square, to: capture, move_type: MoveType::EnPassant });
                }
            }
            // Capture right
            if let Some(capture) = PAWN_RIGHT_CAPTURES[board.side_to_move.idx()][square.idx()] {
                if board.colors[(!board.side_to_move).idx()] & Bitboard::from_square(capture) != Bitboard::EMPTY {
                    pawn_moves.push(Move { from: square, to: capture, move_type: MoveType::Basic });
                }
                else if board.en_passant == Some(capture) {
                    pawn_moves.push(Move { from: square, to: capture, move_type: MoveType::EnPassant });
                }
            }

            // If on promotion rank, convert moves into promotions
            if square.rank() == match board.side_to_move {
                Color::White => Rank::Seven,
                Color::Black => Rank::Two
            } {
                v.extend(pawn_moves.into_iter().flat_map(|mv| Move::promotions(mv.from, mv.to)));
            } else {
                v.extend(pawn_moves);
            }
        }
    }
}

fn gen_attacks(board: &Board, color: Color, blockers: Bitboard) -> Bitboard {
    let mut attacks = Bitboard::EMPTY;
    for piece in PIECES {
        for square in board.pieces[piece.idx()] & board.colors[color.idx()] {
            attacks |= gen_piece_attacks(piece, color, square, blockers);
        }
    }
    attacks
}

fn gen_piece_attacks(piece: Piece, color: Color, square: Square, blockers: Bitboard) -> Bitboard {
    match piece {
        Piece::Rook => magic_tables::get_rook_moves(square, blockers),
        Piece::Knight => KNIGHT_MOVES[square.idx()],
        Piece::Bishop => magic_tables::get_bishop_moves(square, blockers),
        Piece::Queen => magic_tables::get_queen_moves(square, blockers),
        Piece::King => KING_MOVES[square.idx()],
        Piece::Pawn => {
            (match square.forward(color).unwrap().left() {
                Some(square) => Bitboard::from_square(square),
                None => Bitboard::EMPTY
            }) | match square.forward(color).unwrap().right() {
                Some(square) => Bitboard::from_square(square),
                None => Bitboard::EMPTY
            }
        }
    }
}

const KNIGHT_MOVES: [Bitboard; NUM_SQUARES] = {
    let mut knight_moves = [Bitboard::EMPTY; NUM_SQUARES];
    let mut square_idx = 0;
    while square_idx < NUM_SQUARES {
        let square = Square::from_idx(square_idx as u8);
        let mut moves = Bitboard::EMPTY;

        if let Some(step) = square.up() { if let Some(step) = step.up() { if let Some(sq) = step.left() {
            moves.0 |= Bitboard::from_square(sq).0;
        }}}
        if let Some(step) = square.up() { if let Some(step) = step.up() { if let Some(sq) = step.right() {
            moves.0 |= Bitboard::from_square(sq).0;
        }}}
        if let Some(step) = square.down() { if let Some(step) = step.down() { if let Some(sq) = step.left() {
            moves.0 |= Bitboard::from_square(sq).0;
        }}}
        if let Some(step) = square.down() { if let Some(step) = step.down() { if let Some(sq) = step.right() {
            moves.0 |= Bitboard::from_square(sq).0;
        }}}
        if let Some(step) = square.left() { if let Some(step) = step.left() { if let Some(sq) = step.up() {
            moves.0 |= Bitboard::from_square(sq).0;
        }}}
        if let Some(step) = square.left() { if let Some(step) = step.left() { if let Some(sq) = step.down() {
            moves.0 |= Bitboard::from_square(sq).0;
        }}}
        if let Some(step) = square.right() { if let Some(step) = step.right() { if let Some(sq) = step.up() {
            moves.0 |= Bitboard::from_square(sq).0;
        }}}
        if let Some(step) = square.right() { if let Some(step) = step.right() { if let Some(sq) = step.down() {
            moves.0 |= Bitboard::from_square(sq).0;
        }}}

        knight_moves[square_idx] = moves;
        square_idx += 1;
    }

    knight_moves
};

const KING_MOVES: [Bitboard; NUM_SQUARES] = {
    let mut king_moves = [Bitboard::EMPTY; NUM_SQUARES];
    let mut square_idx = 0;
    while square_idx < NUM_SQUARES {
        let square = Square::from_idx(square_idx as u8);
        let mut moves = Bitboard::EMPTY;

        if let Some(step) = square.up() { if let Some(sq) = step.left() {
            moves.0 |= Bitboard::from_square(sq).0;
        }}
        if let Some(sq) = square.up() {
            moves.0 |= Bitboard::from_square(sq).0;
        }
        if let Some(step) = square.up() { if let Some(sq) = step.right() {
            moves.0 |= Bitboard::from_square(sq).0;
        }}
        if let Some(sq) = square.right() {
            moves.0 |= Bitboard::from_square(sq).0;
        }
        if let Some(step) = square.down() { if let Some(sq) = step.right() {
            moves.0 |= Bitboard::from_square(sq).0;
        }}
        if let Some(sq) = square.down() {
            moves.0 |= Bitboard::from_square(sq).0;
        }
        if let Some(step) = square.down() { if let Some(sq) = step.left() {
            moves.0 |= Bitboard::from_square(sq).0;
        }}
        if let Some(sq) = square.left() {
            moves.0 |= Bitboard::from_square(sq).0;
        }

        king_moves[square_idx] = moves;
        square_idx += 1;
    }

    king_moves
};

const fn flip_square_idx(idx: usize) -> usize { idx & 56 }

const PAWN_LEFT_CAPTURES: [[Option<Square>; NUM_SQUARES]; NUM_COLORS] = {
    let mut captures = [[None; NUM_SQUARES]; NUM_COLORS];
    let mut square_idx = 0;
    while square_idx < NUM_SQUARES {
        let square = Square::from_idx(square_idx as u8);
        match square.rank() {
            Rank::One | Rank::Eight => { square_idx += 1; continue },
            _ => ()
        };

        captures[Color::White.idx()][square_idx] = square.up().unwrap().left();
        captures[Color::Black.idx()][square_idx] = square.down().unwrap().left();
        square_idx += 1;
    }
    captures
};

const PAWN_RIGHT_CAPTURES: [[Option<Square>; NUM_SQUARES]; NUM_COLORS] = {
    let mut captures = [[None; NUM_SQUARES]; NUM_COLORS];
    let mut square_idx = 0;
    while square_idx < NUM_SQUARES {
        let square = Square::from_idx(square_idx as u8);
        match square.rank() {
            Rank::One | Rank::Eight => { square_idx += 1; continue },
            _ => ()
        };

        captures[Color::White.idx()][square_idx] = square.up().unwrap().right();
        captures[Color::Black.idx()][square_idx] = square.down().unwrap().right();
        square_idx += 1;
    }
    captures
};