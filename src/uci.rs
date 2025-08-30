use crate::{chess::{self, Board, Move, Piece, Square, START_POS_FEN}, engine};

use std::{sync::mpsc, thread};

#[derive(Debug, PartialEq)]
enum UciCommand {
    Uci,
    SetOption {
        option: UciOption
    },
    Position {
        fen: String,
        moves: Vec<String>
    },
    UciNewGame,
    IsReady,
    Go {
        options: UciGoOptions
    },
    Stop,
    Quit,
}

pub enum HaltCommand {
    Stop,
    Quit
}

#[derive(Debug, PartialEq)]
enum UciOption {

}

#[derive(Debug, PartialEq)]
pub struct UciGoOptions {
    pub search_moves: Option<Vec<String>>,
    pub ponder: bool,
    pub wtime: Option<usize>,
    pub btime: Option<usize>,
    pub winc: Option<usize>,
    pub binc: Option<usize>,
    pub moves_to_go: Option<usize>,
    pub depth: Option<usize>,
    pub nodes: Option<usize>,
    pub mate: Option<usize>,
    pub move_time: Option<usize>,
    pub infinite: bool,
    pub perft: Option<usize>,
}

pub enum UciResponse {
    Uci,
    IsReady,
    BestMove(String),
    Plaintext(String),
}

pub fn run_uci_mode() {
    let (stdin_sender, stdin_receiver) = mpsc::channel();
    let (stdout_sender, stdout_receiver) = mpsc::channel();
    let (halt_sender, halt_receiver) = mpsc::channel();

    // Input thread
    thread::spawn(move || {
        let mut buf = String::new();
        loop {
            buf.clear();
            std::io::stdin()
                .read_line(&mut buf)
                .expect("failed to read line");

            if let Some(command) = parse_uci_command(&buf) {
                match command {
                    UciCommand::Stop => halt_sender.send(HaltCommand::Stop).expect("stdin error"),
                    UciCommand::Quit => halt_sender.send(HaltCommand::Quit).expect("stdin error"),
                    _ => {}
                };
                stdin_sender.send(command).expect("stdin error");
            }
        }
    });

    // Output thread
    thread::spawn(move || {
        for response in stdout_receiver {
            match response {
                UciResponse::Uci => {
                    println!("id name ElleBot");
                    println!("id author Elle");
                    println!("uciok");
                },
                UciResponse::IsReady => {
                    println!("readyok");
                },
                UciResponse::BestMove(mv) => {
                    println!("bestmove {}", mv);
                },
                UciResponse::Plaintext(text) => {
                    println!("{}", text);
                }
            }
        }
    });

    let mut board = Board::default();

    for command in stdin_receiver {
        match command {
            UciCommand::Uci => {
                stdout_sender.send(UciResponse::Uci).expect("stdout error");
            },
            UciCommand::SetOption { option } => {
                todo!()
            },
            UciCommand::Position { fen, moves } => {
                board = match Board::new(&fen) {
                    Some(new) => new,
                    None => return
                };

                for mv in moves {
                    board = chess::make_move(&board, Move::from_uci(&mv, &board).unwrap());
                }
                // println!("debug: set position to {}", board.get_fen());
            },
            UciCommand::UciNewGame => {

            },
            UciCommand::IsReady => {
                stdout_sender.send(UciResponse::IsReady).expect("stdout error");
            },
            UciCommand::Go { options } => {
                println!("debug: received GoOptions {:?}", options);

                // Clear any previous 'stop' commands
                while halt_receiver.try_recv().is_ok() {};

                let search_moves = options.search_moves.as_ref().map(|v| v.iter()
                    .map(|uci| Move::from_uci(uci, &board).unwrap())
                    .collect()
                );

                if options.infinite {
                    println!("debug: searching infinitely");
                    let Ok(Some(best_move)) = engine::search_infinite(&mut board, search_moves, &halt_receiver) else { return; };
                    stdout_sender.send(UciResponse::BestMove(best_move.uci())).expect("stdout error");
                }

                else if let Some(depth) = options.perft {
                    println!("debug: running perft test with depth {}", depth);
                    let count = engine::search_perft(&board, depth, Some(&stdout_sender));
                    stdout_sender.send(UciResponse::Plaintext(count.to_string())).expect("stdout error");
                }

                else {
                    let search_options = engine::decide_options(&mut board, &options);
                    println!("debug: decided search options {:?}", search_options);
                    let Ok(Some(best_move)) = engine::search(&mut board, search_options, search_moves, Some(&halt_receiver)) else { return; };
                    stdout_sender.send(UciResponse::BestMove(best_move.uci())).expect("stdout error");
                }
            },
            UciCommand::Stop => {

            },
            UciCommand::Quit => {
                return;
            },
        };
    }
}

fn parse_uci_command(command: &str) -> Option<UciCommand> {
    let mut words = command.split_whitespace();

    match words.next()? {
        "uci" => Some(UciCommand::Uci),
        "setoption" => {
            todo!()
        },
        "position" => {
            let fen = match words.next()? {
                "startpos" => START_POS_FEN.to_owned(),
                "fen" => (&mut words).take(6).collect::<Vec<&str>>().join(" "),
                _ => return None
            };

            let mut moves = Vec::new();
            if let Some(next) = words.next() {
                if next != "moves" {
                    return Some(UciCommand::Position { fen, moves: Vec::new() });
                }
                moves = words.map(|str| str.to_owned()).collect();
            }

            Some(UciCommand::Position { fen, moves })
        },
        "ucinewgame" => Some(UciCommand::UciNewGame),
        "isready" => Some(UciCommand::IsReady),
        "go" => {
            let mut options = UciGoOptions {
                search_moves: None,
                ponder: false,
                wtime: None,
                btime: None,
                winc: None,
                binc: None,
                moves_to_go: None,
                depth: None,
                nodes: None,
                mate: None,
                move_time: None,
                infinite: false,
                perft: None
            };

            // let mut search_moves = None;
            // let mut ponder = false;
            // let mut wtime = None;
            // let mut btime = None;
            // let mut winc = None;
            // let mut binc = None;
            // let mut moves_to_go = None;
            // let mut depth = None;
            // let mut nodes = None;
            // let mut mate = None;
            // let mut move_time = None;
            // let mut infinite = false;
            // let mut perft = None;

            let mut optionless = true;

            while let Some(param) = words.next() {
                optionless = false;
                match param {
                    "searchmoves" => {
                        options.search_moves = Some(Vec::new());
                        for mv in (&mut words).take_while(|&word| is_uci_move(word)) {
                            options.search_moves.as_mut().unwrap().push(mv.to_owned());
                        }
                    },
                    "ponder" => options.ponder = true,
                    "wtime" => if !parse_next_as_usize(&mut options.wtime, &mut words) { return Some(UciCommand::Go { options }) },
                    "btime" => if !parse_next_as_usize(&mut options.btime, &mut words) { return Some(UciCommand::Go { options }) },
                    "winc" => if !parse_next_as_usize(&mut options.winc, &mut words) { return Some(UciCommand::Go { options }) },
                    "binc" => if !parse_next_as_usize(&mut options.binc, &mut words) { return Some(UciCommand::Go { options }) },
                    "movestogo" => if !parse_next_as_usize(&mut options.moves_to_go, &mut words) { return Some(UciCommand::Go { options }) },
                    "depth" => if !parse_next_as_usize(&mut options.depth, &mut words) { return Some(UciCommand::Go { options }) },
                    "nodes" => if !parse_next_as_usize(&mut options.nodes, &mut words) { return Some(UciCommand::Go { options }) },
                    "mate" => if !parse_next_as_usize(&mut options.mate, &mut words) { return Some(UciCommand::Go { options }) },
                    "movetime" => if !parse_next_as_usize(&mut options.move_time, &mut words) { return Some(UciCommand::Go { options }) },
                    "infinite" => options.infinite = true,
                    "perft" => if !parse_next_as_usize(&mut options.perft, &mut words) { return Some(UciCommand::Go { options }) },
                    _ => return Some(UciCommand::Go { options })
                }
            }

            // If command is "go", execute "go depth 245"
            if optionless {
                options.depth = Some(245);
            }
            
            Some(UciCommand::Go { options })
        },
        "stop" => Some(UciCommand::Stop),
        "quit" => Some(UciCommand::Quit),
        _ => None
    }
}

fn is_uci_move(word: &str) -> bool {
    word.is_ascii()
    && (
        word.len() == 4
        || word.len() == 5 && Piece::from_ascii(word.as_bytes()[4]).is_some()
    )
    && Square::from_san(&word[0..2]).is_some() && Square::from_san(&word[2..4]).is_some()
}

fn parse_next_as_usize<'a>(var: &mut Option<usize>, words: &mut impl Iterator<Item = &'a str>) -> bool {
    if var.is_some() { return false; }
    let Some(word) = words.next() else { return false; };
    let Ok(num) = word.parse::<usize>() else { return false; };
    *var = Some(num);
    true
}