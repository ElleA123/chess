use crate::chess::{Board, COLORS, NUM_COLORS, NUM_FILES, NUM_PIECES, NUM_SQUARES, PIECES};
use crate::prng::PRNG;

const NUM_CASTLES: usize = 16;

pub struct ZobristHasher {
    pieces: [[[u64; NUM_SQUARES]; NUM_PIECES]; NUM_COLORS],
    side_to_move: u64,
    castles: [u64; NUM_CASTLES],
    en_passant: [u64; NUM_FILES],
}

impl ZobristHasher {
    pub const fn new(seed: u128) -> Self {
        let mut prng = PRNG::new(seed);

        let mut pieces = [[[0; NUM_SQUARES]; NUM_PIECES]; NUM_COLORS];

        let mut i = 0;
        while i < NUM_COLORS {
            let mut j = 0;
            while j < NUM_PIECES {
                let mut k = 0;
                while k < NUM_SQUARES {
                    pieces[i][j][k] = prng.next();
                    k += 1;
                }
                j += 1;
            }
            i += 1;
        }

        let side_to_move = prng.next();

        let mut castles = [0; 16];

        let mut i = 0;
        while i < NUM_CASTLES {
            castles[i] = prng.next();
            i += 1;
        }

        let mut en_passant = [0; 8];

        let mut i = 0;
        while i < NUM_FILES {
            en_passant[i] = prng.next();
            i += 1;
        }

        Self { pieces, side_to_move, castles, en_passant }
    }

    pub fn hash(&self, board: &Board) -> u64 {
        let mut hash = 0;

        // Pieces
        for color in COLORS {
            for piece in PIECES {
                for square in board.get_color(color) & board.get_piece(piece) {
                    hash ^= self.pieces[color.idx()][piece.idx()][square.idx()];
                }
            }
        }

        // Side to move
        if board.get_side_to_move().is_white() {
            hash ^= self.side_to_move;
        }

        // Castling
        hash ^= self.castles[board.get_castles().idx()];

        // En passant
        if let Some(c) = board.get_en_passant() {
            hash ^= self.en_passant[c.file().idx()];
        } 

        hash
    }
}