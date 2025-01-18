use std::time::Instant;

const R_STEPS: [(isize, isize); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];
const N_STEPS: [(isize, isize); 8] = [(2, 1), (2, -1), (1, 2), (1, -2), (-1, 2), (-1, -2), (-2, 1), (-2, -1)];
const B_STEPS: [(isize, isize); 4] = [(1, 1), (1, -1), (-1, 1), (-1, -1)];
const KQ_STEPS: [(isize, isize); 8] = [(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (1, -1), (-1, 1), (-1, -1)];

const PROMOTABLES: [PieceType; 4] = [PieceType::Rook, PieceType::Knight, PieceType::Bishop, PieceType::Queen];

const CASTLES: [Move; 4] = [
    Move {
        from: (7, 4),
        to: (7, 6),
        move_type: MoveType::Castle
    },
    Move {
        from: (7, 4),
        to: (7, 2),
        move_type: MoveType::Castle
    },
    Move {
        from: (0, 4),
        to: (0, 6),
        move_type: MoveType::Castle
    },
    Move {
        from: (0, 4),
        to: (0, 2),
        move_type: MoveType::Castle
    }
];

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

#[derive(Debug, Clone, Copy, PartialEq, Hash)]
enum PieceType {
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
    Pawn
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
        String::from(match self {
            &PieceType::Rook => 'r',
            &PieceType::Knight => 'n',
            &PieceType::Bishop => 'b',
            &PieceType::Queen => 'q',
            &PieceType::King => 'k',
            &PieceType::Pawn => 'p',
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Hash)]
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

#[derive(Debug, Clone, PartialEq)]
enum MoveType {
    Basic,
    EnPassant,
    Castle,
    Promotion(PieceType)
}

#[derive(Debug, Clone, PartialEq)]
struct Move {
    from: Coord,
    to: Coord,
    move_type: MoveType
}

impl Move {
    fn new(from: Coord, to: Coord, move_type: MoveType) -> Self {
        Move {
            from,
            to,
            move_type
        }
    }

    fn promotions(from: Coord, to: Coord) -> impl Iterator<Item = Self> {
        PROMOTABLES.iter().map(move |&pt| Move {
            from,
            to,
            move_type: MoveType::Promotion(pt)
        })
    }

    fn uci(&self) -> String {
        let mut uci = format!("{}{}",
            coord_to_string(self.from).unwrap(),
            coord_to_string(self.to).unwrap()
        );
        if let MoveType::Promotion(pt) = self.move_type {
            uci += pt.to_string().as_str();
        }
        uci
    }
}

struct UndoData {
    captured: Option<Piece>,
    en_passant: Option<Coord>,
    allowed_castling: (bool, bool, bool, bool),
    halfmove_count: usize,
}

struct Board {
    board: [[Option<Piece>; 8]; 8],
    side_to_move: bool,
    allowed_castling: (bool, bool, bool, bool), // KQkq
    en_passant: Option<Coord>,
    halfmove_count: usize,
    fullmove_num: usize,
    undo_stack: Vec<UndoData>
}

impl std::fmt::Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut board_str = String::from("\n");
        for row in self.board {
            for cell in row {
                board_str += (match cell {
                    Some(p) => p.to_string(),
                    None => String::from(".")
                } + " ").as_str();
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
                fullmove_num,
                undo_stack: Vec::new()
            })
        } else {
            None
        }
    }

    fn default() -> Self {
        Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap()
    }

    fn fen(&self) -> String {
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

        let mut castling = String::with_capacity(4);
        if self.allowed_castling.0 { castling.push('K'); }
        if self.allowed_castling.1 { castling.push('Q'); }
        if self.allowed_castling.2 { castling.push('k'); }
        if self.allowed_castling.3 { castling.push('q'); }
        if castling == "" { castling.push('-'); }

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

    fn make_move(&mut self, mv: &Move) {
        // Only legal moves should make it to this function
        let (from_y, from_x) = mv.from;
        let (to_y, to_x) = mv.to;
        let piece = self.board[from_y][from_x].unwrap();

        let (captured, is_capture) = match self.board[to_y][to_x] {
            Some(p) => (Some(p), true),
            None => (None, mv.move_type == MoveType::EnPassant)
        };

        // Add data to undo this move
        self.undo_stack.push(UndoData {
            captured,
            en_passant: self.en_passant,
            allowed_castling: self.allowed_castling,
            halfmove_count: self.halfmove_count
        });

        // Update halfmove count
        if piece.piece_type == PieceType::Pawn || is_capture {
            self.halfmove_count = 0;
        } else {
            self.halfmove_count += 1;
        }

        // Make the swap
        self.board[to_y][to_x] = if let MoveType::Promotion(pt) = mv.move_type {
            Some(Piece {
                piece_type: pt,
                is_white: piece.is_white,
            })
        } else {
            Some(piece)
        };
        self.board[from_y][from_x] = None;

        // En Passant
        if mv.move_type == MoveType::EnPassant {
            self.board[from_y][to_x] = None;
        }

        // Castling
        if mv.move_type == MoveType::Castle {
            let (f_x, t_x) = match to_x {
                6 => (7, 5),
                2 => (0, 3),
                _ => panic!("Error: invalid castle")
            };
            let extra_piece = self.board[from_y][f_x].unwrap();
            self.board[to_y][t_x] = Some(extra_piece);
            self.board[from_y][f_x] = None;
        }

        // Update castling availability -- a bit inefficient but like whatevs?
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
        if piece.piece_type == PieceType::Pawn && (if to_y > from_y {to_y - from_y == 2} else {from_y - to_y == 2}) {
            self.en_passant = Some(((to_y as isize - {if piece.is_white {-1} else {1}}) as usize, to_x));
        } else {
            self.en_passant = None;
        }

        // Update fullmove num after black moves
        if !self.side_to_move {self.fullmove_num += 1;}
        // Update turn
        self.side_to_move = !self.side_to_move;
    }

    // fn make_moves(&mut self, moves: Vec<&Move>) {
    //     for mv in moves {
    //         self.make_move(mv);
    //     }
    // }

    fn undo_move(&mut self, mv: &Move) {
        let Some(undo_data) = self.undo_stack.pop() else {return};

        let (from_y, from_x) = mv.from;
        let (to_y, to_x) = mv.to;
        let piece = self.board[to_y][to_x].unwrap();

        // Swap
        self.board[from_y][from_x] = if let MoveType::Promotion(_) = mv.move_type {
            Some(Piece {
                piece_type: PieceType::Pawn,
                is_white: piece.is_white
            })
        } else {
            Some(piece)
        };
        self.board[to_y][to_x] = undo_data.captured;

        if mv.move_type == MoveType::EnPassant {
            self.board[from_y][to_x] = Some(Piece {
                piece_type: PieceType::Pawn,
                is_white: self.side_to_move
            });
        }

        if mv.move_type == MoveType::Castle {
            let (f_x, t_x) = match to_x {
                6 => (7, 5),
                2 => (0, 3),
                _ => panic!("Error: invalid castle")
            };
            let extra_piece = self.board[to_y][t_x].unwrap();
            self.board[from_y][f_x] = Some(extra_piece);
            self.board[to_y][t_x] = None;
        }

        // Update values from saved data
        self.allowed_castling = undo_data.allowed_castling;
        self.en_passant = undo_data.en_passant;
        self.halfmove_count = undo_data.halfmove_count;
        
        // Undo fullmove count
        if self.side_to_move {
            self.fullmove_num -= 1;
        }
        // Update turn
        self.side_to_move = !self.side_to_move;
    }

    // fn undo_moves(&mut self, moves: Vec<&Move>) {
    //     for mv in moves {
    //         self.undo_move(mv);
    //     }
    // }

    fn square_is_color(&self, y: usize, x: usize, color: bool) -> bool {
        match self.board[y][x] {
            Some(piece) => piece.is_white == color,
            None => false
        }
    }

    fn find_players_pieces<'a>(&'a self, color: bool) -> impl Iterator<Item = Coord> + 'a {
        (0..64).map(|i| (i / 8, i % 8))
        .filter(move |&(y, x)| self.square_is_color(y, x, color))
    }

    fn material(&self, color: bool) -> f64 {
        (0..64).map(|i| (i / 8, i % 8))
        .map(|(y, x)| {
            if self.square_is_color(y, x, color) {
                match self.board[y][x].unwrap().piece_type {
                    PieceType::Rook => 5.0,
                    PieceType::Knight => 3.0,
                    PieceType::Bishop => 3.0,
                    PieceType::Queen => 9.0,
                    PieceType::King => 0.0,
                    PieceType::Pawn => 1.0
                }
            } else {
                0.0
            }
        }).sum::<f64>()
    }

    fn score(&self) -> f64 {
        self.material(self.side_to_move) - self.material(!self.side_to_move)
    }

    fn get_linear_moves(&self, y: usize, x: usize, step_list: &[(isize, isize)], one_step_only: bool) -> Vec<Move> {
        let color = self.board[y][x].unwrap().is_white;
        let mut moves = Vec::new();
        for (step_y, step_x) in step_list {
            let mut test_y = (y as isize + step_y) as usize;
            let mut test_x = (x as isize + step_x) as usize;
            while Board::is_on_board(test_y, test_x) {
                if !self.square_is_color(test_y, test_x, color) {
                    moves.push(Move::new((y, x), (test_y, test_x), MoveType::Basic));
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
    fn castling_is_ok(&self, castle: usize, y: usize, x: usize) -> bool {
        let (req_y, allowed, empty) = match castle {
            0 => (7, self.allowed_castling.0, [(7, 5), (7, 5), (7, 6)]), // duplicate items to line up sizes :skull:
            1 => (7, self.allowed_castling.1, [(7, 1), (7, 2), (7, 3)]),
            2 => (0, self.allowed_castling.2, [(0, 5), (0, 5), (0, 6)]),
            3 => (0, self.allowed_castling.3, [(0, 1), (0, 2), (0, 3)]),
            x => panic!("castling_is_ok: illegal `castle` arg: {}", x)
        };
        y == req_y && x == 4 && allowed && empty.into_iter().all(|(y, x)| self.board[y][x].is_none())
    }

    fn get_king_moves(&self, y: usize, x: usize) -> Vec<Move> {
        let mut moves: Vec<Move> = self.get_linear_moves(y, x, &KQ_STEPS, true);

        for castle in 0..4 {
            if self.castling_is_ok(castle, y, x) {
                moves.push(CASTLES[castle].clone());
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
                moves.extend(Move::promotions((y, x), ((y as isize + pawn_dir) as usize, x)));
            } else {
                // Basic move
                moves.push(Move::new((y, x), ((y as isize + pawn_dir) as usize, x), MoveType::Basic));
            }
            // Starting move
            if (color && y == 6) || (!color && y == 1) {
                if self.board[(y as isize + 2*pawn_dir) as usize][x].is_none() {
                    moves.push(Move::new((y, x), ((y as isize + 2*pawn_dir) as usize, x), MoveType::Basic));
                }
            }
        }

        if x != 0 {
            // Capture left
            if self.square_is_color((y as isize + pawn_dir) as usize, x - 1, !color) {
                if will_promote {
                    // Capture left and promote
                    moves.extend(Move::promotions((y, x), ((y as isize + pawn_dir) as usize, x - 1)));
                } else {
                    // Don't promote
                    moves.push(Move::new((y, x), ((y as isize + pawn_dir) as usize, x - 1), MoveType::Basic));
                }
                // En passant left
                if let Some(sq) = self.en_passant {
                    if sq == ((y as isize + pawn_dir) as usize, x - 1) {
                        moves.push(Move::new((y, x), ((y as isize + pawn_dir) as usize, x - 1), MoveType::EnPassant));
                    }
                }
            }
        }
        if x != 7 {
            // Capture right
            if self.square_is_color((y as isize + pawn_dir) as usize, x + 1, !color) {
                if will_promote {
                    // Capture right and promote
                    moves.extend(Move::promotions((y, x), ((y as isize + pawn_dir) as usize, x + 1)));
                } else {
                    // Don't promote
                    moves.push(Move::new((y, x), ((y as isize + pawn_dir) as usize, x + 1), MoveType::Basic));
                }
            }
            // En passant right
            if let Some(sq) = self.en_passant {
                if sq == ((y as isize + pawn_dir) as usize, x + 1) {
                    moves.push(Move::new((y, x), ((y as isize + pawn_dir) as usize, x + 1), MoveType::EnPassant));
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

    fn get_attacks<'a>(&'a self, color: bool) -> impl Iterator<Item = Move> + 'a {
        self.find_players_pieces(color)
        .flat_map(|(y, x)| self.get_piece_moves(y, x))
    }

    fn king_is_attacked(&self, color: bool) -> bool {
        let king = (0..64).into_iter().map(|i| (i / 8, i % 8))
        .find(|&(y, x)|
            match self.board[y][x] {
                Some(piece) => piece.piece_type == PieceType::King && piece.is_white == color,
                None => false
            }
        ).unwrap();

        self.get_attacks(!color)
            .any(|mv| mv.to == king)
    }

    fn get_legal_moves<'a>(&mut self) -> Vec<Move> {
        self.get_attacks(self.side_to_move).collect::<Vec<Move>>().into_iter()
        .filter(|mv| {
            self.make_move(mv);
            let is_legal = !self.king_is_attacked(!self.side_to_move);
            self.undo_move(mv);
            is_legal
        }).collect()
    }

    fn is_check(&self) -> bool {
        self.king_is_attacked(self.side_to_move)
    }
}


fn negamax(board: &mut Board, depth: usize, mut alpha: f64, beta: f64) -> f64 {
    if depth == 0 {
        return board.score();
    }
    let opts = board.get_legal_moves();
    if opts.len() == 0 {
        return if board.is_check() {
            f64::NEG_INFINITY
        } else {
            0.0
        };
    }
    let mut max = f64::NEG_INFINITY;
    for mv in opts {
        board.make_move(&mv);
        let score = -negamax(board, depth - 1, -beta, -alpha);
        board.undo_move(&mv);
        if score > max {
            max = score;
            if max > alpha {
                alpha = score;
                if alpha >= beta {
                    break;
                }
            }
        }
    }
    max
}

fn find_best_move(board: &mut Board, max_depth: usize, window: f64) -> Option<Move> {
    let top_score = board.score();
    let mut best_move = None;
    let mut best_score = f64::NEG_INFINITY;
    for mv in board.get_legal_moves() {
        board.make_move(&mv);
        let score = -negamax(board, max_depth - 1, top_score - window, top_score + window);
        board.undo_move(&mv);
        if score == f64::INFINITY {
            return Some(mv);
        }
        if score > best_score {
            best_score = score;
            best_move = Some(mv);
        }
    }
    best_move
}

fn play_vs_self(depth: usize) {
    let mut board = Board::default();
    loop {
        match find_best_move(&mut board, depth, f64::INFINITY) {
            Some(mv) => {
                println!("{}", mv.uci());
                board.make_move(&mv);
                println!("{}", board);
            },
            None => {
                println!("ggs");
                return;
            }
        }
        if board.halfmove_count >= 100 {
            println!("ggs (50 move rule)");
            return;
        }
    }
}

fn get_input(msg: &str) -> String {
    println!("{}", msg);
    let mut buf = String::new();
    std::io::stdin()
        .read_line(&mut buf)
        .expect("Failed to read line");
    buf.trim().to_owned()
}

fn best_move_of_input() {
    let fen = get_input("Input FEN:");
    let mut board = Board::from_fen(fen.as_str()).unwrap();
    println!("{}", board);

    let Ok(depth) = get_input("Search depth:")
        .parse::<usize>() else { panic!("Error: not a natural number"); };
    if depth == 0 { panic!("Not zero idiot"); }

    let Ok(window) = get_input("Search window:")
        .parse::<f64>() else { panic!("Error: not an f64"); };

    let start = Instant::now();

    let best_move = find_best_move(&mut board, depth, window);

    println!("Time: {:?}", start.elapsed());

    match best_move {
        Some(mv) => println!("{}", mv.uci()),
        None => print!("No moves!")
    }
}

fn main() {
    best_move_of_input();
}