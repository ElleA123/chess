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

const PROMOTABLES: [PieceType; 4] = [PieceType::Rook, PieceType::Knight, PieceType::Bishop, PieceType::Queen];
pub const CASTLES: [Move; 4] = [
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

impl Move {
    pub fn new(from: Coord, to: Coord, move_type: MoveType) -> Self {
        Move {
            from,
            to,
            move_type
        }
    }

    pub fn from(&self) -> Coord {
        self.from
    }

    pub fn to(&self) -> Coord {
        self.to
    }

    pub fn move_type(&self) -> MoveType {
        self.move_type
    }

    pub fn promotions(from: Coord, to: Coord) -> impl Iterator<Item = Self> {
        PROMOTABLES.iter().map(move |&pt| Move {
            from,
            to,
            move_type: MoveType::Promotion(pt)
        })
    }

    pub fn uci(&self) -> String {
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