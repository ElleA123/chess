use std::time::Instant;

const R_STEPS: [(isize, isize); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];
const N_STEPS: [(isize, isize); 8] = [(2, 1), (2, -1), (1, 2), (1, -2), (-1, 2), (-1, -2), (-2, 1), (-2, -1)];
const B_STEPS: [(isize, isize); 4] = [(1, 1), (1, -1), (-1, 1), (-1, -1)];
const KQ_STEPS: [(isize, isize); 8] = [(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (1, -1), (-1, 1), (-1, -1)];

const PROMOTABLES: [PieceType; 4] = [PieceType::Rook, PieceType::Knight, PieceType::Bishop, PieceType::Queen];

enum ParseFenError {
    BoardError,

}

type Coord = (usize, usize);

fn coord_from_str(san: &str) -> Option<Coord> {
    let mut chars = san.chars();
    let x = match chars.next() {
        Some(c) => (c as usize) - ('a' as usize),
        None => { return None; }
    };

    let Some(y) = chars.next() else { return None; };
    let y = match y.to_digit(10) {
        Some(i) => 8 - i as usize,
        None => return None
    };

    if Board::is_on_board(y, x) {
        Some((y, x))
    } else {
        None
    }
}
fn coord_to_string(coord: Coord) -> Option<String> {
    let (y, x) = coord;
    if Board::is_on_board(y, x) {
        Some(format!("{}{}", (x as u8 + 'a' as u8) as char, 8 - y))
    } else {
        None
    }
}

const WHITE: bool = true;
const BLACK: bool = false;

#[derive(Clone, Copy, PartialEq, Debug, Eq, PartialOrd, Ord)]
enum PieceType {
    Pawn,
    King,
    Bishop,
    Knight,
    Rook,
    Queen
}

impl PieceType {
    fn from_char(c: char) -> Option<Self> {
        match c.to_ascii_lowercase() {
            'r' => Some(PieceType::Rook),
            'n' => Some(PieceType::Knight),
            'b' => Some(PieceType::Bishop),
            'q' => Some(PieceType::Queen),
            'k' => Some(PieceType::King),
            'p' => Some(PieceType::Pawn),
            _ => None
        }
    }

    fn to_string(&self) -> String {
        format!("{}", match self {
            &PieceType::Rook => 'r',
            &PieceType::Knight => 'n',
            &PieceType::Bishop => 'b',
            &PieceType::Queen => 'q',
            &PieceType::King => 'k',
            &PieceType::Pawn => 'p',
        })
    }
}

#[derive(Clone, Copy, PartialEq)]
struct Piece {
    piece_type: PieceType,
    is_white: bool
}

impl Piece {
    fn from_char(c: char) -> Option<Self> {
        if let Some(piece_type) = PieceType::from_char(c) {
            Some(Piece {
                piece_type,
                is_white: c.is_ascii_uppercase()
            })
        } else {
            None
        }
    }

    fn to_string(&self) -> String {
        if self.is_white {
            self.piece_type.to_string().to_ascii_uppercase()
        } else {
            self.piece_type.to_string()
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
enum Castle {
    Kingside,
    Queenside
}

#[derive(Debug, PartialEq, Clone)]
struct Move {
    from: Coord,
    to: Coord,
    // is_attack: bool,
    en_passant: bool,
    new_en_passant: bool,
    castle_dir: Option<Castle>, // also for castling
    promotes_to: Option<PieceType>,
}

impl Move {
    fn basic(from: Coord, to: Coord) -> Self {
        Move {
            from,
            to,
            en_passant: false,
            new_en_passant: false,
            castle_dir: None,
            promotes_to: None
        }
    }

    fn uci(&self) -> String {
        format!("{}{}",
            coord_to_string(self.from).unwrap(),
            coord_to_string(self.to).unwrap()
        )
    }

    fn san(&self, board: &Board) -> String {
        // Todo: fix specifying coordinates
        let (f, t) = (self.from, self.to);

        let p = board.board[f.0][f.1].unwrap();
        let piece = match p.piece_type {
            PieceType::Pawn => String::from((f.1 as u8 + 'a' as u8) as char),
            p => p.to_string().to_ascii_uppercase()
        };

        let mut from_spec = String::new();
        let copies = board.find_piece(&p);
        let mut file_added = false;
        let mut rank_added = false;
        for (y, x) in copies {
            if (y, x) == f { continue; }
            if y == f.0 && !file_added {
                from_spec += &((f.1 as u8 + 'a' as u8) as char).to_string();
                file_added = true;
            }
            if x == f.1 && !rank_added {
                from_spec += &(8 - y).to_string();
                rank_added = true;
            }
        }

        let capture = if board.board[t.0][t.1].is_some() {"x"} else {""};
        let dest = coord_to_string((t.0, t.1)).unwrap();
        let promo = if let Some(p) = self.promotes_to {
            format!("={}", &p.to_string())
        } else {
            String::new()
        };
        let result = if board.is_checkmate() {"#"} else if board.is_check() {"+"} else {""};

        format!("{piece}{from_spec}{capture}{dest}{promo}{result}")
    }
}

#[derive(Clone)]
struct Board {
    board: [[Option<Piece>; 8]; 8],
    side_to_move: bool,
    allowed_castling: (bool, bool, bool, bool), // KQkq
    en_passant: Option<Coord>,
    halfmove_count: usize,
    fullmove_num: usize,
    // pieces: Option<Vec<Coord>>,
    // legal_moves: Option<Vec<Move>>,
}

impl std::fmt::Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut board_str = String::from("\n");
        for row in self.board {
            for cell in row {
                board_str += &format!("{} ", (match cell {Some(p) => p.to_string(), None => String::from(".")}));
            }
            board_str += "\n";
        }
        write!(f, "{}", board_str)
    }
}

impl Board {
    fn is_on_board(y: usize, x: usize) -> bool {
        y < 8 &&  x < 8 // type limits cover the bottom half
    }

    fn from_fen(fen: &str) -> Option<Self> {
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
            square => match coord_from_str(square) {
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
                fullmove_num
            })
        } else {
            None
        }
    }

    fn get_fen(&self) -> String {
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

        let mut castling = format!("{}{}{}{}",
            if self.allowed_castling.0 {"K"} else {""},
            if self.allowed_castling.1 {"Q"} else {""},
            if self.allowed_castling.2 {"k"} else {""},
            if self.allowed_castling.3 {"q"} else {""},
        );
        if castling == "" { castling = "-".to_string(); }

        let en_passant = match self.en_passant {
            Some(c) => coord_to_string(c).unwrap(),
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

    fn default() -> Self {
        Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap()
    }

    fn make_move(&mut self, mv: &Move) {
        // Only legal moves should make it to this function
        let (from_y, from_x) = mv.from;
        let (to_y, to_x) = mv.to;

        let piece = self.board[from_y][from_x].unwrap();

        // Update halfmove count
        if piece.piece_type == PieceType::Pawn || self.board[to_y][to_x].is_some() {
            self.halfmove_count = 0;
        } else {
            self.halfmove_count += 1;
        }

        // Make the swap
        self.board[to_y][to_x] = match mv.promotes_to {
            Some(p) => Some(Piece {
                piece_type: p,
                is_white: piece.is_white,
            }),
            None => Some(piece)};
        self.board[from_y][from_x] = None;

        // En passant
        if mv.en_passant {
            self.board[from_y][to_x] = None;
        }

        // Castling
        if let Some(dir) = &mv.castle_dir {
            let (f_x, t_x) = match dir {
                Castle::Kingside => (7, 5),
                Castle::Queenside => (0, 3)
            };
            let extra_piece = self.board[from_y][f_x].unwrap();
            self.board[to_y][t_x] = Some(extra_piece);
            self.board[from_y][f_x] = None;
        }

        // Update castling availability
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
        self.en_passant = if mv.new_en_passant { Some(((to_y as isize - {if piece.is_white {-1} else {1}}) as usize, to_x)) } else {None};
        // Update fullmove num
        if !self.side_to_move {self.fullmove_num += 1;}
        // Update turn
        self.side_to_move = !self.side_to_move;
    }

    // fn make_moves(&mut self, moves: &Vec<Move>) {
    //     for mv in moves {
    //         self.make_move(mv);
    //     }
    // }

    fn square_is_color(&self, y: usize, x: usize, color: bool) -> bool {
        match self.board[y][x] {
            Some(piece) => piece.is_white == color,
            None => false
        }
    }

    fn find_players_pieces(&self, color: bool) -> Vec<Coord> {
        (0..64).map(|i| (i / 8, i % 8))
        .filter(|&(y, x)| self.square_is_color(y, x, color))
        .collect()
    }

    fn find_piece(&self, piece: &Piece) -> Vec<Coord> {
        (0..64).map(|i| (i / 8, i % 8)).filter(|&(y, x)| {
            match self.board[y][x] {
                Some(p) => &p == piece,
                None => false
            }
        }).collect()
    }

    fn get_linear_moves(&self, y: usize, x: usize, step_list: &[(isize, isize)], one_step_only: bool) -> Vec<Move> {
        let color = self.board[y][x].unwrap().is_white;
        let mut moves = Vec::new();
        for (step_y, step_x) in step_list {
            let mut test_y = (y as isize + step_y) as usize;
            let mut test_x = (x as isize + step_x) as usize;
            while Board::is_on_board(test_y, test_x) {
                // Empty: add move and continue
                // Opposite color: add move and stop
                // Same color: stop w/o adding move
                if !self.square_is_color(test_y, test_x, color) {
                    moves.push(Move::basic((y, x), (test_y, test_x)));
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
    fn castling_is_ok(&self, castle: usize) -> bool {
        let (allowed, empty) = match castle {
            0 => (self.allowed_castling.0, [(7, 5), (7, 5), (7, 6)],), // [(7, 4), (7, 5), (7, 6)]), // duplicate items to line up sizes :skull:
            1 => (self.allowed_castling.1, [(7, 1), (7, 2), (7, 3)],), // [(7, 2), (7, 3), (7, 4)]),
            2 => (self.allowed_castling.2, [(0, 5), (0, 5), (0, 6)],), // [(0, 4), (0, 5), (0, 6)]),
            3 => (self.allowed_castling.3, [(0, 1), (0, 2), (0, 3)],), // [(0, 2), (0, 3), (0, 4)]),
            x => panic!("castling_is_ok: illegal `castle` arg: {}", x)
        };
        allowed && empty.into_iter().all(|(y, x)| self.board[y][x].is_none())
        // && not_attacked.into_iter().all(|c| !squares_attacked.contains(&c))
    }

    fn get_king_moves(&self, y: usize, x: usize) -> Vec<Move> {
        // let color = self.board[y][x].unwrap().is_white;

        // let squares_attacked: Vec<Coord> = if check_not_attacked {
        //     self.get_attacks(!color).into_iter()
        //         .map(|mv| mv.to).collect()
        // } else {
        //     Vec::new()
        // };
        
        let mut moves: Vec<Move> = self.get_linear_moves(y, x, &KQ_STEPS, true);
            // .into_iter().filter(|mv| !squares_attacked.contains(&mv.to)).collect();

        if (y, x) == (7, 4) {
            if self.castling_is_ok(0) {
                moves.push(Move {
                    from: (7, 4),
                    to: (7, 6),
                    en_passant: false,
                    new_en_passant: false,
                    castle_dir: Some(Castle::Kingside),
                    promotes_to: None,
                });
            }
            if self.castling_is_ok(1) {
                moves.push(Move {
                    from: (7, 4),
                    to: (7, 2),
                    en_passant: false,
                    new_en_passant: false,
                    castle_dir: Some(Castle::Queenside),
                    promotes_to: None,
                });
            }
        }
        if (y, x) == (0, 4) {
            if self.castling_is_ok(2) {
                moves.push(Move {
                    from: (0, 4),
                    to: (0, 6),
                    en_passant: false,
                    new_en_passant: false,
                    castle_dir: Some(Castle::Kingside),
                    promotes_to: None,
                });
            }
            if self.castling_is_ok(3) {
                moves.push(Move {
                    from: (0, 4),
                    to: (0, 2),
                    en_passant: false,
                    new_en_passant: false,
                    castle_dir: Some(Castle::Queenside),
                    promotes_to: None,
                });
            }
        }
        moves
    }

    fn get_pawn_moves(&self, y: usize, x: usize) -> Vec<Move> {
        let color = self.board[y][x].unwrap().is_white;
        let pawn_dir = if color {-1} else {1};
        let will_promote = (y as isize + pawn_dir) == {if color {0} else {7}};
        let mut moves = Vec::new();
        if self.board[(y as isize + pawn_dir) as usize][x].is_none() {
            if will_promote {
                // Promotion moves
                for pt in PROMOTABLES {
                    moves.push(Move {
                        from: (y, x),
                        to: ((y as isize + pawn_dir) as usize, x),
                        en_passant: false,
                        new_en_passant: false,
                        castle_dir: None,
                        promotes_to: Some(pt),
                    });
                }
            } else {
                // Basic move
                moves.push(Move::basic((y, x), ((y as isize + pawn_dir) as usize, x)));
            }
            // Starting move
            if (color && y == 6) || (!color && y == 1) {
                if self.board[(y as isize + 2*pawn_dir) as usize][x].is_none() {
                    moves.push(Move {
                        from: (y, x),
                        to: ((y as isize + 2*pawn_dir) as usize, x),
                        en_passant: false,
                        new_en_passant: true,
                        castle_dir: None,
                        promotes_to: None,
                    });
                }
            }
        }

        if x != 0 {
            // Capture left
            if self.square_is_color((y as isize + pawn_dir) as usize, x - 1, !color) {
                if will_promote {
                    // Capture left and promote
                    for pt in [PieceType::Rook, PieceType::Knight, PieceType::Bishop, PieceType::Queen] {
                        moves.push(Move {
                            from: (y, x),
                            to: ((y as isize + pawn_dir) as usize, x - 1),
                            en_passant: false,
                            new_en_passant: false,
                            promotes_to: Some(pt),
                            castle_dir: None,
                        });
                    }
                } else {
                    // Don't promote
                    moves.push(Move::basic((y, x), ((y as isize + pawn_dir) as usize, x - 1)));
                }
                // En passant left
                if let Some(sq) = self.en_passant {
                    if sq == ((y as isize + pawn_dir) as usize, x - 1) {
                        moves.push(Move {
                            from: (y, x),
                            to: ((y as isize + pawn_dir) as usize, x - 1),
                            en_passant: true,
                            new_en_passant: false,
                            castle_dir: None,
                            promotes_to: None,
                        });
                    }
                }
            }
        }
        if x != 7 {
            // Capture right
            if self.square_is_color((y as isize + pawn_dir) as usize, x + 1, !color) {
                if will_promote {
                    // Capture right and promote
                    for pt in PROMOTABLES {
                        moves.push(Move {
                            from: (y, x),
                            to: ((y as isize + pawn_dir) as usize, x),
                            en_passant: false,
                            new_en_passant: false,
                            castle_dir: None,
                            promotes_to: Some(pt),
                        });
                    }
                } else {
                    // Don't promote
                    moves.push(Move::basic((y, x), ((y as isize + pawn_dir) as usize, x + 1)));
                }
            }
            // En passant right
            if let Some(sq) = self.en_passant {
                if sq == ((y as isize + pawn_dir) as usize, x + 1) {
                    moves.push(Move {
                        from: (y, x),
                        to: ((y as isize + pawn_dir) as usize, x + 1),
                        en_passant: true,
                        new_en_passant: false,
                        promotes_to: None,
                        castle_dir: None,
                    });
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

    fn get_attacks(&self, color: bool) -> Vec<Move> {
        self.find_players_pieces(color).into_iter()
        .flat_map(|(y, x)| self.get_piece_moves(y, x))
        .collect()
    }

    fn king_is_attacked(&self, color: bool) -> bool {
        let king = self.find_piece(&Piece {
            piece_type: PieceType::King,
            is_white: color
        })[0];

        self.get_attacks(!color).into_iter()
            .any(|mv| mv.to == king)
    }

    /*
    Checkmate
    - King is attacked
      - Find my king
      - Get their attacks
        - Find their pieces
        - Get all of their moves -- TODO: remove pawn non-attacks
        - return
      - Check if any attack lands on king square
      - return
    - Get legal moves
      - Find my pieces
      - Get all of my moves
      - Filter by:
        - Make the move
        - Is king attacked
    To find legal moves:
    - Their pieces and attacks
    - My pieces and moves
      - Their pieces and attacks for each move (different board = no duplicated lookup)
    - No short-circuiting possible, entire space must be searched

    */

    fn get_legal_moves(&self) -> Vec<Move> {
        self.get_attacks(self.side_to_move)
        .into_iter().filter(|mv| {
            let mut test_board = self.clone();
            test_board.make_move(mv);
            !test_board.king_is_attacked(self.side_to_move)
        }).collect()
    }

    fn is_check(&self) -> bool {
        self.king_is_attacked(self.side_to_move)
    }

    fn is_checkmate(&self) -> bool {
        self.is_check() && self.get_legal_moves().len() == 0
    }

    fn is_stalemate(&self) -> bool {
        !self.is_check() && self.get_legal_moves().len() == 0
    }
}


fn is_mate_in_n(board: &Board, depth: usize, my_move: bool) -> bool {
    let mut moves = board.get_legal_moves();
    moves.sort_by(|mv1, mv2|
        board.board[mv2.from.0][mv2.from.1].unwrap().piece_type.cmp(&board.board[mv1.from.0][mv1.from.1].unwrap().piece_type)
    );

    if !my_move && (board.is_check() && moves.len() == 0) {
        return true;
    }
    if moves.len() == 0 || depth == 0 {
        return false;
    }

    if my_move {
        moves.into_iter().any(|mv| {
            let mut test_board = board.clone();
            test_board.make_move(&mv);
            is_mate_in_n(&test_board, depth - 1, !my_move)
        })
    } else {
        moves.into_iter().all(|mv| {
            let mut test_board = board.clone();
            test_board.make_move(&mv);
            is_mate_in_n(&test_board, depth, !my_move)
        })
    }
}

fn find_mate_within_n(board: &Board, max_depth: usize) -> Option<Move> {
    let mut moves = board.get_legal_moves();
    moves.sort_by(|mv1, mv2|
        board.board[mv2.from.0][mv2.from.1].unwrap().piece_type.cmp(&board.board[mv1.from.0][mv1.from.1].unwrap().piece_type)
    );
    for mv in &moves {
        let mut test_board = board.clone();
        test_board.make_move(mv);
        println!("{}", mv.uci());
        if is_mate_in_n(&test_board, max_depth - 1, false) {
            return Some(mv.clone());
        }
    }
    None
}

// fn find_mate_within_n(board: &Board, max_depth: usize) -> Option<Move> {
//     // Iteratively-deepening depth-first search, slow
//     let moves = board.get_legal_moves();
//     for depth in 0..max_depth+1 {
//         for mv in &moves {
//             let mut test_board = board.clone();
//             test_board.make_move(mv);
//             if is_mate_in_n(&test_board, depth - 1, false) {
//                 return Some(mv.clone());
//             }
//         }
//     }
//     None
// }

fn get_input(msg: &str) -> String {
    println!("{}", msg);
    let mut buf = String::new();
    std::io::stdin()
        .read_line(&mut buf)
        .expect("Failed to read line");
    buf.trim().to_owned()
}

fn main() {
    // let fen = get_input("Input FEN:");
    let fen = "r4rk1/p7/2Q3p1/2Pp4/6p1/N1P4P/P4qPK/R4R2 b - - 0 1";
    let mut board = Board::from_fen(fen).unwrap();
    // let board = Board::default();

    println!("{}", board);

    println!("{}", board.get_fen());
    

    // for _ in 0..5 {
    //     let mv = &board.get_legal_moves()[0];
    //     println!("{}", mv.san(&board));
    //     board.make_move(mv);
    // }
    // println!("{}", board);
    // println!("{}", board.get_fen());

    // let depth = get_input("Search depth:");
    // let Ok(depth) = depth.parse::<usize>() else { panic!("Error: not a natural number"); };
    let depth = 3;

    let start = Instant::now();

    let arb_mate = find_mate_within_n(&board, depth);

    println!("Time: {:?}", start.elapsed());

    match arb_mate {
        Some(mv) => println!("{}", mv.uci()),
        None => println!("No mate")
    }
}