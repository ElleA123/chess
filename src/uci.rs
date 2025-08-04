use std::{thread, sync::mpsc};

use crate::{chess::{Board, Move, PieceType, Coord}, engine::get_best_move};

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
        search_moves: Vec<String>,
        ponder: bool,
        wtime: Option<usize>,
        btime: Option<usize>,
        winc: Option<usize>,
        binc: Option<usize>,
        moves_to_go: Option<usize>,
        depth: Option<usize>,
        nodes: Option<usize>,
        mate: Option<usize>,
        move_time: Option<usize>,
        infinite: bool,
        perft: Option<usize>,
    },
    Stop,
    Quit,
}

#[derive(Debug, PartialEq)]
enum UciOption {

}

enum UciResponse {
    Uci,
    IsReady,
    BestMove(String),
    Info,
}

pub fn run_uci_mode() {
    let (stdin_sender, stdin_receiver) = mpsc::channel();
    let (stdout_sender, stdout_receiver) = mpsc::channel();

    // Input thread
    thread::spawn(move || {
        let mut buf = String::new();
        loop {
            buf.clear();
            std::io::stdin()
                .read_line(&mut buf)
                .expect("failed to read line");

            if let Some(command) = parse_uci_command(&buf) {
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
                _ => todo!()
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
                
            },
            UciCommand::Position { fen, moves } => {
                board = Board::from_fen(&fen).unwrap();
                for mv in moves {
                    board.make_move(&Move::from_uci(&mv, &board).unwrap(), false);
                }
                println!("debug: set position to {}", board.get_fen());
            },
            UciCommand::UciNewGame => {

            },
            UciCommand::IsReady => {
                stdout_sender.send(UciResponse::IsReady).expect("stdout error");
            },
            UciCommand::Go {
                search_moves,
                ponder,
                wtime,
                btime,
                winc,
                binc,
                moves_to_go,
                depth,
                nodes,
                mate,
                move_time,
                infinite,
                perft,
            } => {
                let max_depth = depth.unwrap_or(usize::MAX).min(5);
                let best_move = get_best_move(&mut board, max_depth).unwrap();
                stdout_sender.send(UciResponse::BestMove(best_move.uci())).expect("stdout error");
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
                "startpos" => "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_owned(),
                "fen" => (&mut words).take(6).collect::<Vec<&str>>().join(" "),
                _ => return None
            };

            let mut moves = Vec::new();
            if let Some(next) = words.next() {
                if next != "moves" { return None; }
                moves = words.map(|str| str.to_owned()).collect();
            }

            Some(UciCommand::Position {
                fen,
                moves
            })
        },
        "ucinewgame" => Some(UciCommand::UciNewGame),
        "isready" => Some(UciCommand::IsReady),
        "go" => {
            let mut search_moves: Vec<String> = Vec::new();
            let mut ponder = false;
            let mut wtime = None;
            let mut btime = None;
            let mut winc = None;
            let mut binc = None;
            let mut moves_to_go = None;
            let mut depth = None;
            let mut nodes = None;
            let mut mate = None;
            let mut move_time = None;
            let mut infinite = false;
            let mut perft = None;

            let mut optionless = true;

            while let Some(param) = words.next() {
                optionless = false;
                match param {
                    "searchmoves" => {
                        for mv in (&mut words).take_while(|&word| is_uci_move(word)) {
                            search_moves.push(mv.to_owned());
                        }
                    },
                    "ponder" => ponder = true,
                    "wtime" => parse_next_as_usize(&mut wtime, &mut words)?,
                    "btime" => parse_next_as_usize(&mut btime, &mut words)?,
                    "winc" => parse_next_as_usize(&mut winc, &mut words)?,
                    "binc" => parse_next_as_usize(&mut binc, &mut words)?,
                    "movestogo" => parse_next_as_usize(&mut moves_to_go, &mut words)?,
                    "depth" => parse_next_as_usize(&mut depth, &mut words)?,
                    "nodes" => parse_next_as_usize(&mut nodes, &mut words)?,
                    "mate" => parse_next_as_usize(&mut mate, &mut words)?,
                    "movetime" => parse_next_as_usize(&mut move_time, &mut words)?,
                    "infinite" => infinite = true,
                    "perft" => parse_next_as_usize(&mut perft, &mut words)?,
                    _ => return None
                }
            }

            // If command is "go", execute "go depth 245"
            if optionless {
                depth = Some(245);
            }
            
            Some(UciCommand::Go {
                search_moves,
                ponder,
                wtime,
                btime,
                winc,
                binc,
                moves_to_go,
                depth,
                nodes,
                mate,
                move_time,
                infinite,
                perft
            })
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
        || word.len() == 5 && PieceType::from_ascii_char(word.as_bytes()[4]).is_some()
    )
    && Coord::from_san(&word[0..2]).is_some() && Coord::from_san(&word[2..4]).is_some()
}

fn parse_next_as_usize<'a>(var: &mut Option<usize>, words: &mut impl Iterator<Item = &'a str>) -> Option<()> {
    if var.is_some() { return None; }
    let Ok(num) = words.next()?.parse::<usize>() else { return None; };
    *var = Some(num);
    Some(())
}