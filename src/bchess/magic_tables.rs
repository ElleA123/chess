use std::sync::{LazyLock, OnceLock};

use rand::{RngCore, SeedableRng, rngs::SmallRng};

use crate::bchess::{bitboard::Bitboard, square::{Square, NUM_SQUARES}};
use crate::prng::PRNG;

// https://analog-hors.github.io/site/magic-bitboards/

pub fn get_rook_moves(square: Square, blockers: Bitboard) -> Bitboard {
    let entry = &ROOK_MAGICS.get().unwrap()[square.idx()];
    entry.1[magic_table_idx(&entry.0, blockers)]
}

pub fn get_bishop_moves(square: Square, blockers: Bitboard) -> Bitboard {
    let entry = &BISHOP_MAGICS.get().unwrap()[square.idx()];
    entry.1[magic_table_idx(&entry.0, blockers)]
}

pub fn get_queen_moves(square: Square, blockers: Bitboard) -> Bitboard {
    get_rook_moves(square, blockers) | get_bishop_moves(square, blockers)
}

static ROOK_MAGICS: OnceLock<[(Magic, Vec<Bitboard>); NUM_SQUARES]> = OnceLock::new();
static BISHOP_MAGICS: OnceLock<[(Magic, Vec<Bitboard>); NUM_SQUARES]> = OnceLock::new();

pub fn init_tables() {
    ROOK_MAGICS.set({
        let mut magics = core::array::from_fn(|_|
            (Magic {
                mask: Bitboard::EMPTY,
                mult: 0,
                idx_bits: 0
            },
            Vec::with_capacity(1 << ROOK_IDX_BITS))
        );

        let mut rng = SmallRng::seed_from_u64(123123);

        let mut square_idx = 0;
        while square_idx < NUM_SQUARES {
            let square = Square::from_idx(square_idx as u8);
            let mask = ROOK_MASKS[square_idx];

            'search: loop {
                let mult = rng.next_u64() & rng.next_u64() & rng.next_u64(); 
                let magic = Magic { mask, mult, idx_bits: 64 - ROOK_IDX_BITS };

                let mut moves_table = vec![Bitboard::EMPTY; 1 << ROOK_IDX_BITS];

                let mut blockers = Bitboard::EMPTY;
                loop {
                    let moves = rook_moves(square, blockers);

                    // Check if entry matches, or write entry to table
                    let entry = &mut moves_table[magic_table_idx(&magic, blockers)];
                    if entry.0 == Bitboard::EMPTY.0 {
                        *entry = moves;
                    } else if entry.0 != moves.0 {
                        continue 'search;
                    }

                    // Move to next subset
                    blockers.0 = blockers.0.wrapping_sub(mask.0) & mask.0;
                    if blockers.0 == Bitboard::EMPTY.0 {
                        break;
                    }
                }

                magics[square_idx] = (magic, moves_table);
                square_idx += 1;
                break;
            }
        }

        magics
    }).map_err(|_| ()).expect("error initializing rook magics");
    BISHOP_MAGICS.set({
        let mut magics = core::array::from_fn(|_|
            (Magic {
                mask: Bitboard::EMPTY,
                mult: 0,
                idx_bits: 0
            },
            Vec::with_capacity(1 << BISHOP_IDX_BITS))
        );

        let mut square_idx = 0;
        while square_idx < NUM_SQUARES {
            let square = Square::from_idx(square_idx as u8);
            let mask = BISHOP_MASKS[square_idx];

            let mut prng = PRNG::new(123123);

            'search: loop {
                let mult = prng.next() & prng.next() & prng.next();
                let magic = Magic { mask, mult, idx_bits: 64 - BISHOP_IDX_BITS };

                let mut moves_table = vec![Bitboard::EMPTY; 1 << BISHOP_IDX_BITS];

                let mut blockers = Bitboard::EMPTY;
                loop {
                    let moves = bishop_moves(square, blockers);

                    // Check if entry matches, or write entry to table
                    let entry = &mut moves_table[magic_table_idx(&magic, blockers)];
                    if entry.0 == Bitboard::EMPTY.0 {
                        *entry = moves;
                    } else if entry.0 != moves.0 {
                        continue 'search;
                    }

                    // Move to next subset
                    blockers.0 = blockers.0.wrapping_sub(mask.0) & mask.0;
                    if blockers.0 == Bitboard::EMPTY.0 {
                        break;
                    }
                }

                magics[square_idx] = (magic, moves_table);
                square_idx += 1;
                break;
            }
        }

        magics
    }).map_err(|_| ()).expect("error initializing bishop magics");
}

#[derive(Debug, Clone, Copy)]
struct Magic {
    mask: Bitboard,
    mult: u64,
    idx_bits: u8
}

const fn magic_table_idx(magic: &Magic, blockers: Bitboard) -> usize {
    let blockers = blockers.0 & magic.mask.0;
    let hash = blockers.wrapping_mul(magic.mult);
    let idx = (hash >> magic.idx_bits) as usize;
    idx
}

const ROOK_IDX_BITS: u8 = 12;

const ROOK_MASKS: [Bitboard; NUM_SQUARES] = {
    let mut masks = [Bitboard::EMPTY; 64];

    let mut square_idx = 0;
    while square_idx < NUM_SQUARES {
        let square = Square::from_idx(square_idx as u8);
        let mut mask = Bitboard::EMPTY;

        if let Some(mut sq) = square.up() {
            loop {
                let next = match sq.up() {
                    Some(next) => next,
                    None => break
                };

                mask.0 |= Bitboard::from_square(sq).0;

                sq = next;
            }
        }

        if let Some(mut sq) = square.down() {
            loop {
                let next = match sq.down() {
                    Some(next) => next,
                    None => break
                };

                mask.0 |= Bitboard::from_square(sq).0;

                sq = next;
            }
        }

        if let Some(mut sq) = square.left() {
            loop {
                let next = match sq.left() {
                    Some(next) => next,
                    None => break
                };

                mask.0 |= Bitboard::from_square(sq).0;

                sq = next;
            }
        }

        if let Some(mut sq) = square.right() {
            loop {
                let next = match sq.right() {
                    Some(next) => next,
                    None => break
                };

                mask.0 |= Bitboard::from_square(sq).0;

                sq = next;
            }
        }

        masks[square_idx] = mask;
        square_idx += 1;
    }

    masks
};

const BISHOP_IDX_BITS: u8 = 9;

const BISHOP_MASKS: [Bitboard; NUM_SQUARES] = {
    let mut masks = [Bitboard::EMPTY; 64];

    let mut square_idx = 0;
    while square_idx < NUM_SQUARES {
        let square = Square::from_idx(square_idx as u8);
        let mut mask = Bitboard::EMPTY;

        if let Some(step) = square.up() {
        if let Some(mut sq) = step.left() {
            loop {
                let next = match sq.up() {
                    Some(step) => match step.left() {
                        Some(next) => next,
                        None => break
                    },
                    None => break
                };

                mask.0 |= Bitboard::from_square(sq).0;

                sq = next;
            }
        }}

        if let Some(step) = square.up() {
        if let Some(mut sq) = step.right() {
            loop {
                let next = match sq.up() {
                    Some(step) => match step.right() {
                        Some(next) => next,
                        None => break
                    },
                    None => break
                };

                mask.0 |= Bitboard::from_square(sq).0;

                sq = next;
            }
        }}

        if let Some(step) = square.down() {
        if let Some(mut sq) = step.left() {
            loop {
                let next = match sq.down() {
                    Some(step) => match step.left() {
                        Some(next) => next,
                        None => break
                    },
                    None => break
                };

                mask.0 |= Bitboard::from_square(sq).0;

                sq = next;
            }
        }}

        if let Some(step) = square.down() {
        if let Some(mut sq) = step.right() {
            loop {
                let next = match sq.down() {
                    Some(step) => match step.right() {
                        Some(next) => next,
                        None => break
                    },
                    None => break
                };

                mask.0 |= Bitboard::from_square(sq).0;

                sq = next;
            }
        }}

        masks[square_idx] = mask;
        square_idx += 1;
    }

    masks
};

const fn rook_moves(square: Square, blockers: Bitboard) -> Bitboard {
    let mut moves = Bitboard::EMPTY;

    let mut sq = square;
    loop {
        sq = match sq.up() {
            Some(sq) => sq,
            None => break
        };

        let move_bb = Bitboard::from_square(sq);
        moves.0 |= move_bb.0;

        if move_bb.0 & blockers.0 != Bitboard::EMPTY.0 {
            break;
        }
    }

    sq = square;
    loop {
        sq = match sq.down() {
            Some(sq) => sq,
            None => break
        };

        let move_bb = Bitboard::from_square(sq);
        moves.0 |= move_bb.0;

        if move_bb.0 & blockers.0 != Bitboard::EMPTY.0 {
            break;
        }
    }

    sq = square;
    loop {
        sq = match sq.left() {
            Some(sq) => sq,
            None => break
        };

        let move_bb = Bitboard::from_square(sq);
        moves.0 |= move_bb.0;

        if move_bb.0 & blockers.0 != Bitboard::EMPTY.0 {
            break;
        }
    }

    sq = square;
    loop {
        sq = match sq.right() {
            Some(sq) => sq,
            None => break
        };

        let move_bb = Bitboard::from_square(sq);
        moves.0 |= move_bb.0;

        if move_bb.0 & blockers.0 != Bitboard::EMPTY.0 {
            break;
        }
    }

    moves
}

const fn bishop_moves(square: Square, blockers: Bitboard) -> Bitboard {
    let mut moves = Bitboard::EMPTY;

    let mut sq = square;
    loop {
        sq = match sq.up() {
            Some(step) => match step.left() {
                Some(sq) => sq,
                None => break
            },
            None => break
        };

        let move_bb = Bitboard::from_square(sq);
        moves.0 |= move_bb.0;

        if move_bb.0 & blockers.0 != Bitboard::EMPTY.0 {
            break;
        }
    }

    sq = square;
    loop {
        sq = match sq.up() {
            Some(step) => match step.right() {
                Some(sq) => sq,
                None => break
            },
            None => break
        };

        let move_bb = Bitboard::from_square(sq);
        moves.0 |= move_bb.0;

        if move_bb.0 & blockers.0 != Bitboard::EMPTY.0 {
            break;
        }
    }

    sq = square;
    loop {
        sq = match sq.down() {
            Some(step) => match step.left() {
                Some(sq) => sq,
                None => break
            },
            None => break
        };

        let move_bb = Bitboard::from_square(sq);
        moves.0 |= move_bb.0;

        if move_bb.0 & blockers.0 != Bitboard::EMPTY.0 {
            break;
        }
    }

    sq = square;
    loop {
        sq = match sq.down() {
            Some(step) => match step.right() {
                Some(sq) => sq,
                None => break
            },
            None => break
        };

        let move_bb = Bitboard::from_square(sq);
        moves.0 |= move_bb.0;

        if move_bb.0 & blockers.0 != Bitboard::EMPTY.0 {
            break;
        }
    }

    moves
}