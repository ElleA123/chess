use std::{thread, sync::mpsc};

use crate::chess::{Board, Move};
use crate::engine::get_best_move;

pub fn setup_uci_engine() {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let mut buf = String::new();
        loop {
            buf.clear();
            std::io::stdin()
                .read_line(&mut buf)
                .expect("failed to read line");

            tx.send(buf.clone()).unwrap();
        }
    });

    let mut board: Board = Board::default();
    for cmd in rx {
        parse_uci_command(&cmd, &mut board);
    }
}

fn parse_uci_command(line: &str, board: &mut Board) {
    let words: Vec<&str> = line.trim().split(" ").collect();
    match words[0] {
        "quit" => std::process::exit(0),
        "uci" => identify_self(),
        "setoption" => set_option(words),
        "position" => set_position(words, board),
        "ucinewgame" => (),
        "isready" => println!("readyok"),
        "go" => parse_and_go(words, board),
        "stop" => (),
        _ => ()
    }
}

fn identify_self() {
    println!("id name ElleBot\n id author Elle");
}

fn set_option(words: Vec<&str>) {

}

fn set_position(words: Vec<&str>, board: &mut Board) {
    *board = if words[1] == "startpos" {
        Board::default()
    } else {
        Board::from_fen(words[1]).unwrap()
    };

    if words.len() == 2 { return; }

    for mv in words.into_iter().skip(2) {
        board.make_move(&Move::from_uci(mv, board), false);
    }
}

fn parse_and_go(words: Vec<&str>, board: &mut Board) {
    get_and_send_move(board, 4);
    // if words.len() == 1 {
    //     get_and_send_move(board, 4); // 245 once timing is implemented
    // }

    // let mut idx = 1;
    // while idx < words.len() {
    //     let param = words[idx];
    //     match param {
    //         "infinite" => 
    //     }
    // }

}

fn get_and_send_move(board: &mut Board, depth: u32) {
    if let Some(mv) = get_best_move(board, depth) {
        println!("{}", mv.get_uci())
    } else {
        println!("0000");
    }    
}