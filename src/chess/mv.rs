use super::coord::Coord;
use super::piece::PieceType;
use super::board::Board;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MoveType {
    Basic,
    EnPassant,
    Castle,
    Promotion(PieceType)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Move {
    pub from: Coord,
    pub to: Coord,
    pub move_type: MoveType
}

impl Move {
    pub const fn new(from: Coord, to: Coord, move_type: MoveType) -> Self {
        Move { from, to, move_type }
    }

    pub fn from_uci(uci: &str, board: &Board) -> Option<Self> {
        if !uci.is_ascii() || uci.len() < 4 { return None; }

        let from = Coord::from_san(&uci[0..2])?;
        let to = Coord::from_san(&uci[2..4])?;

        let move_type = match board.get_square(from).unwrap().piece_type {
            PieceType::Pawn => {
                if let Some(ep) = board.get_en_passant(){
                    if to == ep { MoveType::EnPassant } else { MoveType::Basic }
                }
                else if to.y == 0 || to.y == 7 {
                    MoveType::Promotion(PieceType::from_char(uci.chars().nth(4)?)?)
                }
                else { MoveType::Basic }
            },
            PieceType::King => {
                if uci == "e1g1" || uci == "e1c1" || uci == "e8g8" || uci == "e8c8" { MoveType::Castle }
                else { MoveType::Basic }
            },
            _ => MoveType::Basic
        };

        Some( Self { from, to, move_type } )
    }

    pub const fn promotions(from: Coord, to: Coord) -> [Self; 4] {
        [Move {from, to, move_type: MoveType::Promotion(PieceType::Rook)},
         Move {from, to, move_type: MoveType::Promotion(PieceType::Knight)},
         Move {from, to, move_type: MoveType::Promotion(PieceType::Bishop)},
         Move {from, to, move_type: MoveType::Promotion(PieceType::Queen)}]
    }

    pub fn uci(&self) -> String {
        format!("{}{}{}",
            self.from.to_string(),
            self.to.to_string(),
            if let MoveType::Promotion(piece_type) = self.move_type {
                piece_type.as_str()
            } else {
                ""
            }
        )

        // let mut uci = format!("{}{}",
        //     self.from.to_string(),
        //     self.to.to_string()
        // );
        // if let MoveType::Promotion(pt) = self.move_type {
        //     uci += pt.as_str();
        // }
        // uci
    }
}