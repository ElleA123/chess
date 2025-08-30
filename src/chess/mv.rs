use super::{board::Board, piece::Piece, square::{Rank, Square}};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MoveType {
    Basic,
    EnPassant,
    Castle,
    FirstPawnMove,
    Promotion(Piece)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Move {
    pub from: Square,
    pub to: Square,
    pub move_type: MoveType
}

impl Move {
    #[inline]
    pub const fn new(from: Square, to: Square, move_type: MoveType) -> Self {
        Move { from, to, move_type }
    }

    pub fn from_uci(uci: &str, board: &Board) -> Option<Self> {
        if !uci.is_ascii() || uci.len() < 4 { return None; }

        let from = Square::from_san(&uci[0..2])?;
        let to = Square::from_san(&uci[2..4])?;

        let move_type = match board.get_piece_at(from)? {
            Piece::Pawn => {
                if let Some(ep) = board.get_en_passant() {
                    if to == ep { MoveType::EnPassant } else { MoveType::Basic }
                }
                else if to.rank() == Rank::One || to.rank() == Rank::Eight {
                    MoveType::Promotion(Piece::from_ascii(uci.bytes().nth(4)?)?)
                }
                else if from.rank() == Rank::Two && to.rank() == Rank::Four
                     || from.rank() == Rank::Seven && to.rank() == Rank::Five {
                    MoveType::FirstPawnMove
                }
                else { MoveType::Basic }
            },
            Piece::King => {
                if uci == "e1g1" || uci == "e1c1" || uci == "e8g8" || uci == "e8c8" { MoveType::Castle }
                else { MoveType::Basic }
            },
            _ => MoveType::Basic
        };

        Some( Self { from, to, move_type } )
    }

    pub fn uci(&self) -> String {
        format!("{}{}{}",
            self.from.to_string(),
            self.to.to_string(),
            if let MoveType::Promotion(piece) = self.move_type {
                piece.to_string()
            } else {
                String::new()
            }
        )
    }

    #[inline]
    pub const fn promotions(from: Square, to: Square) -> [Self; 4] {
        [Move {from, to, move_type: MoveType::Promotion(Piece::Rook)},
         Move {from, to, move_type: MoveType::Promotion(Piece::Knight)},
         Move {from, to, move_type: MoveType::Promotion(Piece::Bishop)},
         Move {from, to, move_type: MoveType::Promotion(Piece::Queen)}]
    }
}

impl std::fmt::Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.uci())
    }
}