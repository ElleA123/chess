use crate::coord::Coord;
use crate::PieceType;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MoveType {
    Basic,
    EnPassant,
    Castle,
    Promotion(PieceType)
}

#[derive(Debug, Clone, PartialEq)]
pub struct Move {
    from: Coord,
    to: Coord,
    move_type: MoveType
}

// const PROMOTABLES: [PieceType; 4] = [PieceType::Rook, PieceType::Knight, PieceType::Bishop, PieceType::Queen];
pub const CASTLES: [Move; 4] = [
    Move { from: Coord::new(7, 4), to: Coord::new(7, 6), move_type: MoveType::Castle },
    Move { from: Coord::new(7, 4), to: Coord::new(7, 2), move_type: MoveType::Castle },
    Move { from: Coord::new(0, 4), to: Coord::new(0, 6), move_type: MoveType::Castle },
    Move { from: Coord::new(0, 4), to: Coord::new(0, 2), move_type: MoveType::Castle }
];

impl Move {
    pub const fn new(from: Coord, to: Coord, move_type: MoveType) -> Self {
        Move { from, to, move_type }
    }

    pub const fn get_from(&self) -> &Coord {
        &self.from
    }

    pub const fn get_to(&self) -> &Coord {
        &self.to
    }

    pub const fn get_move_type(&self) -> &MoveType {
        &self.move_type
    }

    pub const fn get_promotions(from: Coord, to: Coord) -> [Self; 4] {
        [Move {from, to, move_type: MoveType::Promotion(PieceType::Rook)},
         Move {from, to, move_type: MoveType::Promotion(PieceType::Knight)},
         Move {from, to, move_type: MoveType::Promotion(PieceType::Bishop)},
         Move {from, to, move_type: MoveType::Promotion(PieceType::Queen)},]
    }

    pub fn get_uci(&self) -> String {
        let mut uci = format!("{}{}",
            self.from.to_string(),
            self.to.to_string()
        );
        if let MoveType::Promotion(pt) = self.move_type {
            uci += pt.to_string().to_ascii_uppercase().as_str();
        }
        uci
    }
}