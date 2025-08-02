use std::mem::MaybeUninit;

use crate::chess::Board;

pub struct ZobristHasher {
    pieces: [[[u64; 8]; 8]; 12],
    side_to_move: u64,
    allowed_castling: [u64; 16],
    en_passant: [u64; 8],
}

impl ZobristHasher {
    pub fn new(seed: u128) -> Self {
        let mut prng = PRNG::new(seed);
        let mut pieces = Box::new([[[MaybeUninit::uninit(); 8]; 8]; 12]);
        for i in 0..12 {
            for j in 0..8 {
                for k in 0..8 {
                    pieces[i][j][k].write(prng.next());
                }
            }
        }

        let side_to_move = prng.next();

        let mut allowed_castling = [MaybeUninit::uninit(); 16];
        for i in 0..16 {
            allowed_castling[i].write(prng.next());
        }

        let mut en_passant = [MaybeUninit::uninit(); 8];
        for i in 0..8 {
            en_passant[i].write(prng.next());
        }

        Self {
            pieces: *(unsafe { std::mem::transmute::<Box<[[[MaybeUninit<u64>; 8]; 8]; 12]>, Box<[[[u64; 8]; 8]; 12]>>(pieces) }),
            side_to_move,
            allowed_castling: unsafe { std::mem::transmute(allowed_castling) },
            en_passant: unsafe { std::mem::transmute(en_passant) }
        }
    }

    pub fn hash(&self, board: &Board) -> u64 {
        let mut hash = 0;

        // Pieces
        let position = board.get_board();
        for y in 0..8 {
            for x in 0..8 {
                if let Some(piece) = position[y][x] {
                    let p = 6 * piece.color as usize + piece.piece_type as usize;
                    hash ^= self.pieces[p][y][x];
                }
            }
        }

        // Side to move
        if board.get_side_to_move().is_white() {
            hash ^= self.side_to_move;
        }

        // Castling
        let castling = board.get_allowed_castling();
        let castling_idx = castling.0 as usize
            + ((castling.1 as usize) << 1)
            + ((castling.2 as usize) << 2)
            + ((castling.3 as usize) << 3);
        hash ^= self.allowed_castling[castling_idx];

        // En passant
        if let Some(c) = board.get_en_passant() {
            hash ^= self.en_passant[c.x];
        } 

        hash
    }
}

struct PRNG(u128);

impl PRNG {
    fn new(seed: u128) -> Self {
        Self(seed)
    }

    fn next(&mut self) -> u64 {
        // Constants from https://en.wikipedia.org/wiki/Linear_congruential_generator#Parameters_in_common_use
        self.0 *= 6364136223846793005;
        self.0 += 1442695040888963407;
        self.0 &= (1 << 64) - 1;
        return self.0 as u64;
    }
}