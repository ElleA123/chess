#[derive(Debug)]
enum Error {
    ParsePieceError,
    ParseFenError,
    CoordError,
}

type Coord = (usize, usize);

fn coord_from_str(san: &str) -> Result<Coord, Error> {
    let mut chars = san.chars();
    let file = match chars.next() {
        Some(c) => (c as usize) - ('a' as usize),
        None => { return Err(Error::CoordError) }
    };

    let Some(rank) = chars.next() else { return Err(Error::CoordError); };
    let rank = match rank.to_digit(10) {
        Some(i) => 8 - i as usize,
        None => return Err(Error::CoordError)
    };

    Ok((rank, file))
}

const WHITE: bool = true;
const BLACK: bool = false;

#[derive(Clone, Copy, PartialEq, Debug)]
enum PieceType {
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
    Pawn
}

impl PieceType {
    pub fn from_char(c: char) -> Result<Self, Error> {
        match c.to_ascii_lowercase() {
            'r' => Ok(PieceType::Rook),
            'n' => Ok(PieceType::Knight),
            'b' => Ok(PieceType::Bishop),
            'q' => Ok(PieceType::Queen),
            'k' => Ok(PieceType::King),
            'p' => Ok(PieceType::Pawn),
            _ => Err(Error::ParsePieceError)
        }
    }

    pub fn to_string(&self) -> String {
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
    pub fn from_char(c: char) -> Result<Self, Error> {
        if let Ok(piece_type) = PieceType::from_char(c) {
            Ok(Piece {
                piece_type,
                is_white: c.is_ascii_uppercase()
            })
        } else {
            Err(Error::ParsePieceError)
        }
    }

    pub fn to_string(&self) -> String {
        if self.is_white {
            self.piece_type.to_string().to_ascii_uppercase()
        } else {
            self.piece_type.to_string()
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
struct Move {
    from: Coord,
    to: Coord,
    extra_move: Option<(Coord, Coord)>, // also for castling
    extra_capture: Option<Coord>, // for en passant
    new_en_passant: Option<Coord>,
    promotes_to: Option<PieceType>,
}

#[derive(Clone)]
struct Position {
    board: [[Option<Piece>; 8]; 8],
    side_to_move: bool,
    allowed_castling: (bool, bool, bool, bool), // KQkq
    en_passant: Option<Coord>,
}

impl std::fmt::Display for Position {
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

impl Position {
    pub fn is_on_board(y: usize, x: usize) -> bool {
        y < 8 &&  x < 8 // type limits cover the bottom half
    }

    fn from_fen(fen: String) -> Result<Self, Error> {
        let mut fen_fields = fen.split(" ");

        // Position
        let Some(pieces) = fen_fields.next() else { return Err(Error::ParseFenError) };
        let mut board: [[Option<Piece>; 8]; 8] = [[None; 8]; 8];

        // TODO: check for repeated numbers (e.g. "44") in fen
        let mut y = 0;
        for rank in pieces.split("/") {
            if y >= 8 { return Err(Error::ParseFenError); }

            let mut x = 0;
            for p in rank.chars() {
                if x >= 8 { return Err(Error::ParseFenError); }

                if let Ok(piece) = Piece::from_char(p) {
                    board[y][x] = Some(piece);
                    x += 1;
                }
                else if p.is_ascii_digit() && p != '0' {
                    x += p.to_digit(10).unwrap() as usize;
                }
                else {
                    return Err(Error::ParseFenError);
                }
            }
            if x != 8 { return Err(Error::ParseFenError); }
            y += 1;
        }
        if y != 8 { return Err(Error::ParseFenError); }

        // Player to move
        let Some(side_to_move) = fen_fields.next() else { return Err(Error::ParseFenError) };
        let side_to_move = match side_to_move {
            "w" => WHITE,
            "b" => BLACK,
            _ => return Err(Error::ParseFenError)
        };

        // Castling avilability - TODO: add error handling
        let Some(allowed_castling) = fen_fields.next() else { return Err(Error::ParseFenError) };
        let allowed_castling = (
            allowed_castling.contains("K"),
            allowed_castling.contains("Q"),
            allowed_castling.contains("k"),
            allowed_castling.contains("q"),
        );

        // En passant
        let Some(en_passant) = fen_fields.next() else { return Err(Error::ParseFenError) };
        let en_passant = match en_passant {
            "-" => None,
            square => match coord_from_str(square) {
                Ok(c) => Some(c),
                Err(_) => { return Err(Error::ParseFenError); }
            }
        };

        // Halfmove count and fullmove count aren't stored
        // if fen_fields.count() == 2 {
        Ok(Position {
            board,
            side_to_move,
            allowed_castling,
            en_passant,
        })
        // } else {
        //     Err(Error::ParseFenError)
        // }
    }

    pub fn default() -> Self {
        Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_owned()).unwrap()
    }

    fn make_move(&mut self, mv: &Move) {
        // Only legal moves should make it to this function
        let (from_y, from_x) = mv.from;
        let (to_y, to_x) = mv.to;

        // Make the swap
        let piece = self.board[from_y][from_x].unwrap();
        self.board[to_y][to_x] = match mv.promotes_to {
            Some(p) => Some(Piece {
                piece_type: p,
                is_white: piece.is_white,
            }),
            None => Some(piece)};
        self.board[from_y][from_x] = None;

        // Extra capture (en passant)
        if let Some((y, x)) = mv.extra_capture {
            self.board[y][x] = None;
        }

        // Extra piece move (castling)
        if let Some(((f_y, f_x), (t_y, t_x))) = mv.extra_move {
            let extra_piece = self.board[f_y][f_x].unwrap();
            self.board[t_y][t_x] = Some(extra_piece);
            self.board[f_y][f_x] = None;
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
        self.en_passant = mv.new_en_passant;
        // Update turn
        self.side_to_move = !self.side_to_move;
    }

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

    fn get_linear_moves(&self, y: usize, x: usize, step_list: Vec<(i32, i32)>, one_step_only: bool) -> Vec<Move> {
        let color = self.board[y][x].unwrap().is_white;
        let mut moves = Vec::new();
        for (step_y, step_x) in step_list {
            let mut test_y = (y as i32 + step_y) as usize;
            let mut test_x = (x as i32 + step_x) as usize;
            while Position::is_on_board(test_y, test_x) {
                // Empty: add move and continue
                // Opposite color: add move and stop
                // Same color: stop w/o adding move
                if !self.square_is_color(test_y, test_x, color) {
                    moves.push(Move {
                        from: (y, x),
                        to: (test_y, test_x),
                        extra_move: None,
                        extra_capture: None,
                        new_en_passant: None,
                        promotes_to: None,
                    });
                    if self.square_is_color(test_y, test_x, !color) {
                        break;
                    }
                } else {
                    break;
                }
                
                test_y = (test_y as i32 + step_y) as usize;
                test_x = (test_x as i32 + step_x) as usize;

                if one_step_only {
                    break;
                }
            }
        }
        moves
    }

    fn get_rook_moves(&self, y: usize, x: usize) -> Vec<Move> {
        self.get_linear_moves(y, x, vec![(1, 0), (-1, 0), (0, 1), (0, -1)], false)
    }
    fn get_knight_moves(&self, y: usize, x: usize) -> Vec<Move> {
        self.get_linear_moves(
            y, x, vec![(2, 1), (2, -1), (1, 2), (1, -2), (-1, 2), (-1, -2), (-2, 1), (-2, -1)], true
        )
    }
    fn get_bishop_moves(&self, y: usize, x: usize) -> Vec<Move> {
        self.get_linear_moves(y, x, vec![(1, 1), (1, -1), (-1, 1), (-1, -1)], false)
    }
    fn get_queen_moves(&self, y: usize, x: usize) -> Vec<Move> {
        self.get_linear_moves(
            y, x, vec![(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (1, -1), (-1, 1), (-1, -1)], false
        )
    }
    fn castling_is_ok(&self, castle: usize, squares_attacked: &Vec<Coord>) -> bool {
        let (right, empty, not_attacked) = match castle {
            0 => (self.allowed_castling.0, [(7, 5), (7, 5), (7, 6)], [(7, 4), (7, 5), (7, 6)]), // duplicate items to line up sizes :skull:
            1 => (self.allowed_castling.1, [(7, 1), (7, 2), (7, 3)], [(7, 2), (7, 3), (7, 4)]),
            2 => (self.allowed_castling.2, [(0, 5), (0, 5), (0, 6)], [(0, 4), (0, 5), (0, 6)]),
            3 => (self.allowed_castling.3, [(0, 1), (0, 2), (0, 3)], [(0, 2), (0, 3), (0, 4)]),
            x => panic!("castling_is_ok: illegal `castle` arg: {}", x)
        };
        right && empty.into_iter().all(|(y, x)| self.board[y][x].is_none())
        && not_attacked.into_iter().all(|c| !squares_attacked.contains(&c))
    }

    fn get_king_moves(&self, y: usize, x: usize, check_not_attacked: bool) -> Vec<Move> {
        let color = self.board[y][x].unwrap().is_white;

        let squares_attacked: Vec<Coord> = if check_not_attacked {
            self.get_all_attacks(!color).into_iter()
                .map(|mv| mv.to).collect()
        } else {
            Vec::new()
        };
        
        let mut moves: Vec<Move> = self.get_linear_moves(
            y, x, vec![(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (1, -1), (-1, 1), (-1, -1)], true
        ).into_iter().filter(|mv| !squares_attacked.contains(&mv.to)).collect();

        if (y, x) == (7, 4) {
            if self.castling_is_ok(0, &squares_attacked) {
                // println!("K");
                moves.push(Move {
                    from: (7, 4),
                    to: (7, 6),
                    extra_move: Some(((7, 7), (7, 5))),
                    extra_capture: None,
                    new_en_passant: None,
                    promotes_to: None,
                });
            }
            if self.castling_is_ok(1, &squares_attacked) {
                // println!("Q");
                moves.push(Move {
                    from: (7, 4),
                    to: (7, 2),
                    extra_move: Some(((7, 0), (7, 3))),
                    extra_capture: None,
                    new_en_passant: None,
                    promotes_to: None,
                });
            }
        }
        if (y, x) == (0, 4) {
            if self.castling_is_ok(2, &squares_attacked) {
                // println!("k");
                moves.push(Move {
                    from: (0, 4),
                    to: (0, 6),
                    extra_move: Some(((0, 7), (0, 5))),
                    extra_capture: None,
                    new_en_passant: None,
                    promotes_to: None,
                });
            }
            if self.castling_is_ok(3, &squares_attacked) {
                // println!("q");
                moves.push(Move {
                    from: (0, 4),
                    to: (0, 2),
                    extra_move: Some(((0, 0), (0, 3))),
                    extra_capture: None,
                    new_en_passant: None,
                    promotes_to: None,
                });
            }
        }
        moves
    }

    fn get_pawn_moves(&self, y: usize, x: usize) -> Vec<Move> {
        let color = self.board[y][x].unwrap().is_white;
        let pawn_dir = if color {-1} else {1};
        let mut moves = Vec::new();
        if y == 0 || y == 8 {
            println!("there is a pawn where a pawn should NOT be");
            println!("{}", self);
        }
        if self.board[(y as i32 + pawn_dir) as usize][x].is_none() {
            if (y as i32 + pawn_dir) == {if color {0} else {7}} {
                // Promotion moves
                for pt in [PieceType::Rook, PieceType::Knight, PieceType::Bishop, PieceType::Queen] {
                    moves.push(Move {
                        from: (y, x),
                        to: ((y as i32 + pawn_dir) as usize, x),
                        extra_move: None,
                        extra_capture: None,
                        new_en_passant: None,
                        promotes_to: Some(pt),
                    });
                }
            } else {
                // Basic move
                moves.push(Move {
                    from: (y, x),
                    to: ((y as i32 + pawn_dir) as usize, x),
                    extra_move: None,
                    extra_capture: None,
                    new_en_passant: None,
                    promotes_to: None,
                });
            }
            // Starting move
            if (color && y == 6) || (!color && y == 1) {
                if self.board[(y as i32 + 2*pawn_dir) as usize][x].is_none() {
                    moves.push(Move {
                        from: (y, x),
                        to: ((y as i32 + 2*pawn_dir) as usize, x),
                        extra_move: None,
                        extra_capture: None,
                        new_en_passant: Some(((y as i32 + 2*pawn_dir) as usize, x)),
                        promotes_to: None,
                    });
                }
            }
        }

        if x != 0 {
            // Capture left
            if self.square_is_color((y as i32 + pawn_dir) as usize, x - 1, !color) {
                moves.push(Move {
                    from: (y, x),
                    to: ((y as i32 + pawn_dir) as usize, x - 1),
                    extra_move: None,
                    extra_capture: None,
                    new_en_passant: None,
                    promotes_to: None,
                });
                // En passant left
                if let Some(sq) = self.en_passant {
                    if sq == (y, x - 1) {
                        moves.push(Move {
                            from: (y, x),
                            to: ((y as i32 + pawn_dir) as usize, x - 1),
                            extra_move: None,
                            extra_capture: Some((y, x - 1)),
                            new_en_passant: None,
                            promotes_to: None,
                        });
                    }
                }
            }
        }
        if x != 7 {
            // Capture right
            if self.square_is_color((y as i32 + pawn_dir) as usize, x + 1, !color) {
                moves.push(Move {
                    from: (y, x),
                    to: ((y as i32 + pawn_dir) as usize, x + 1),
                    extra_move: None,
                    extra_capture: None,
                    new_en_passant: None,
                    promotes_to: None,
                });
            }
            // En passant right
            if let Some(sq) = self.en_passant {
                if sq == (y, x + 1) {
                    moves.push(Move {
                        from: (y, x),
                        to: ((y as i32 + pawn_dir) as usize, x + 1),
                        extra_move: None,
                        extra_capture: Some((y, x + 1)),
                        new_en_passant: None,
                        promotes_to: None,
                    });
                }
            }
        }
        moves
    }

    fn get_piece_moves(&self, y: usize, x: usize, check_not_attacked: bool) -> Vec<Move> {
        let piece = self.board[y][x].unwrap();
        match piece.piece_type {
            PieceType::Rook => self.get_rook_moves(y, x),
            PieceType::Knight => self.get_knight_moves(y, x),
            PieceType::Bishop => self.get_bishop_moves(y, x),
            PieceType::Queen => self.get_queen_moves(y, x),
            PieceType::King => self.get_king_moves(y, x, check_not_attacked),
            PieceType::Pawn => self.get_pawn_moves(y, x),
        }
    }

    fn get_all_attacks(&self, color: bool) -> Vec<Move> {
        self.find_players_pieces(color).into_iter()
        .flat_map(|(y, x)| self.get_piece_moves(y, x, false))
        // .filter(|mv| mv.extra_move.is_none())
        .collect()
    }

    fn get_all_moves(&self, color: bool) -> Vec<Move> {
        self.find_players_pieces(color).into_iter()
        .flat_map(|(y, x)| self.get_piece_moves(y, x, true))
        .collect()
    }

    fn get_legal_moves(&self) -> Vec<Move> {
        self.find_players_pieces(self.side_to_move).into_iter()
        .flat_map(|(y, x)| self.get_piece_moves(y, x, true))
        .filter(|mv| {
            let mut test_board = self.clone();
            // println!("{:?}", mv);
            test_board.make_move(mv);
            !test_board.king_is_attacked(self.side_to_move)
        }).collect()
    }

    fn king_is_attacked(&self, color: bool) -> bool {
        let king = self.find_piece(&Piece {
            piece_type: PieceType::King,
            is_white: color
        })[0];

        self.get_all_moves(!color).into_iter()
            .any(|mv| mv.to == king)
    }

    fn is_check(&self) -> bool {
        self.king_is_attacked(self.side_to_move)
    }

    fn is_checkmate(&self) -> bool {
        // let is_king_attacked = self.king_is_attacked(self.side_to_move);
        // let legal_moves = self.get_legal_moves();
        // println!("King attacked: {}", is_king_attacked);
        // println!("# of legal moves: {}", legal_moves.len());
        // for mv in &legal_moves {
        //     println!("{:?}", mv);
        // }
        // is_king_attacked && legal_moves.len() == 0
        self.is_check() && self.get_legal_moves().len() == 0
    }

    fn is_stalemate(&self) -> bool {
        !self.is_check() && self.get_legal_moves().len() == 0
    }
}

fn is_mate_in_n(board: &Position, depth: usize, my_move: bool) -> bool {
    if !my_move && board.is_checkmate() {
        return true;
    }
    else if board.get_legal_moves().len() == 0 || depth == 0 {
        return false;
    }

    if my_move {
        board.get_legal_moves().into_iter().any(|mv| {
            let mut test_board = board.clone();
            // println!("MY TURN: {:?}", mv);
            test_board.make_move(&mv);
            is_mate_in_n(&test_board, depth, !my_move)
        })
    } else {
        board.get_legal_moves().into_iter().all(|mv| {
            let mut test_board = board.clone();
            // println!("THEIR TURN: {:?}", mv);
            test_board.make_move(&mv);
            is_mate_in_n(&test_board, depth - 1, !my_move)
        })
    }
}

// fn is_mate_in_n(board: &Position, depth: usize) -> bool {
//     // Shouldn't be depth-first
//     if depth == 0 {
//         return board.is_checkmate();
//     }
//     for i in 0..depth+1 {
//         if is_mate_in_exactly_n(board, i, false) {
//             return true;
//         }
//     }
//     false
// }

fn find_mate_within_n(board: &Position, max_depth: usize) -> Option<Move> {
    for depth in 0..max_depth+1 {
        for mv in board.get_legal_moves() {
            let mut test_board = board.clone();
            test_board.make_move(&mv);
            if is_mate_in_n(&test_board, depth, false) {
                return Some(mv);
            }
        }
    }
    None
}

// fn find_mate_in_ones(board: &Position) -> Vec<Move> {
//     board.get_legal_moves().into_iter().filter(|mv| {
//         let mut test_board = board.clone();
//         test_board.make_move(mv);
//         test_board.is_checkmate()
//     }).collect()
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
    let fen = get_input("Input FEN:");
    let board = Position::from_fen(fen).unwrap();
    println!("{}", board);

    use std::time::Instant;
    let start = Instant::now();

    let depth = get_input("Input depth:");
    let Ok(depth) = depth.parse::<usize>() else { panic!("Error: not a natural number"); };
    let arb_mate = find_mate_within_n(&board, depth);

    println!("Time: {:?}", start.elapsed());

    match arb_mate {
        Some(mv) => println!("{:?}", mv),
        None => println!("No mate")
    }

    // let castle = Move {
    //     from: (7, 4),
    //     to: (7, 6),
    //     extra_move: Some(((7, 7), (7, 5))),
    //     extra_capture: None,
    //     new_en_passant: None,
    //     promotes_to: None
    // };
    // board.make_move(&castle);
    // println!("{}", board);
}