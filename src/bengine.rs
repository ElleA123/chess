use crate::bchess::{board::{self, Board}, color::Color, mv::Move, piece::{Piece, PIECES}};
use crate::uci::{HaltCommand, UciGoOptions};

use std::{collections::HashMap, sync::mpsc, time::Instant};

mod psts;

const MAX_DEPTH: usize = 6;
const MAX_TIME: usize = usize::MAX; // ms

const fn next_iter_time_guess(depth: usize) -> usize {
    match depth {
        1 => 0,
        2 => 5,
        3 => 50,
        4 => 250,
        5 => 1500,
        6 => 2500,
        _ => usize::MAX
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SearchOptions {
    pub max_depth: usize,
    pub time: usize,
    pub nodes: Option<usize>,
}

pub fn decide_options(board: &mut Board, go_options: &UciGoOptions) -> SearchOptions {
    let time;
    if let Some(move_time) = go_options.move_time {
        time = move_time;
    }
    else if let Some(clock_time) = match board.get_side_to_move() {
        Color::White => go_options.wtime,
        Color::Black => go_options.btime
    } {
        let increment = match board.get_side_to_move() {
            Color::White => go_options.winc,
            Color::Black => go_options.binc
        }.unwrap_or_default();

        // https://www.chessprogramming.org/Time_Management#Time_Controls
        time = clock_time / 20 + increment / 2;
    }
    else {
        time = MAX_TIME;
    }

    let time_bound_depth = {
        let mut depth = 0;
        let mut total_time: usize = 0;
        loop {
            depth += 1;
            total_time = total_time.saturating_add(next_iter_time_guess(depth));
            if total_time >= time {
                break;
            }
        }
        depth - 1
    };
    let max_depth = go_options.depth.unwrap_or(MAX_DEPTH).min(time_bound_depth).min(MAX_DEPTH);

    let nodes = go_options.nodes;

    SearchOptions {
        max_depth,
        time,
        nodes,
    }
}

pub fn perft(board: &Board, max_depth: usize, depth: usize, map: Option<&HashMap<String, usize>>) -> usize {
    if depth == 0 { return 1; }

    let mut count = 0;

    let mut moves = Vec::new();
    board::gen_legal_moves(board, &mut moves);

    // if depth == 1 { return moves.len(); }

    for mv in moves {
        let subtotal = perft(&board::make_move(board, mv), max_depth, depth - 1, map);

        if depth == max_depth {
            println!("{}: {}", mv.uci(), subtotal)
        };

        count += subtotal;
    }

    count
}

pub fn search_infinite(board: &Board, search_moves: Option<Vec<Move>>, halt_receiver: &mpsc::Receiver<HaltCommand>) -> Result<Option<Move>, ()> {
    let mut moves = search_moves.unwrap_or_else(|| {
        let mut moves = Vec::new();
        board::gen_legal_moves(board, &mut moves);
        moves
    });
    let mut best_move = None;
    let mut depth = 1;

    loop {
        // Check for a halt command
        if let Ok(halt_cmd) = halt_receiver.try_recv() {
            match halt_cmd {
                HaltCommand::Stop => return Ok(best_move),
                HaltCommand::Quit => return Err(())
            }
        }

        // Search
        let result = dfs_search_and_sort(board, &mut moves, &mut best_move, depth, Some(halt_receiver));
        // Check for a halt command while searching
        if let Err(halt_command) = result {
            match halt_command {
                HaltCommand::Stop => return Ok(best_move),
                HaltCommand::Quit => return Err(())
            }
        }

        depth += 1;
    }
}

pub fn search(
    board: &Board, options: SearchOptions, search_moves: Option<Vec<Move>>, halt_receiver: Option<&mpsc::Receiver<HaltCommand>>
) -> Result<Option<Move>, ()> {
    // Search for the best move in a position using [iterative deepening](https://www.chessprogramming.org/Iterative_Deepening)
    // If `halt_receiver` is `Some(rx)`, the search can end early if a `HaltCommand` is sent to the receiver. 
    let start_time = Instant::now();

    let SearchOptions { max_depth, time, nodes } = options;

    let mut moves = search_moves.unwrap_or_else(|| {
        let mut moves = Vec::new();
        board::gen_legal_moves(board, &mut moves);
        moves
    });

    let mut best_move: Option<Move> = None;

    for depth in 1..max_depth {
        // Check for a halt command
        if let Some(halt_receiver) = halt_receiver {
            if let Ok(halt_cmd) = halt_receiver.try_recv() {
                match halt_cmd {
                    HaltCommand::Stop => return Ok(best_move),
                    HaltCommand::Quit => return Err(())
                }
            }
        }

        // Check if we have time to do a search at this depth
        if time.saturating_sub(start_time.elapsed().as_millis() as usize) < next_iter_time_guess(depth) {
            return Ok(best_move);
        }

        // Search
        let result = dfs_search_and_sort(board, &mut moves, &mut best_move, depth, halt_receiver);
        // Check for a halt command while searching
        if let Err(halt_command) = result {
            match halt_command {
                HaltCommand::Stop => return Ok(best_move),
                HaltCommand::Quit => return Err(())
            }
        }
    }

    if time.saturating_sub(start_time.elapsed().as_millis() as usize) < next_iter_time_guess(max_depth) {
        return Ok(best_move);
    }

    // Check for a halt command
    if let Some(halt_receiver) = halt_receiver {
        if let Ok(halt_cmd) = halt_receiver.try_recv() {
            match halt_cmd {
                HaltCommand::Stop => return Ok(best_move),
                HaltCommand::Quit => return Err(())
            }
        }
    }

    // Final search
    let result = dfs_search_final(board, &mut moves, &mut best_move, max_depth, halt_receiver);
    // Check for a halt command while searching
    if let Err(halt_command) = result {
        match halt_command {
            HaltCommand::Stop => return Ok(best_move),
            HaltCommand::Quit => return Err(())
        }
    }

    Ok(best_move)
}

fn dfs_search_and_sort(
    board: &Board, moves: &mut Vec<Move>, best_move: &mut Option<Move>, depth: usize, halt_receiver: Option<&mpsc::Receiver<HaltCommand>>
) -> Result<(), HaltCommand> {
    // Run depth-first search with a max depth of `depth` and sort `moves` from worst to best.
    // The function also updates `best_move` as soon as a better move is discovered; combined with move-sorting from previous iterations,
    // this means that `best_move` will have a reasonable move at any sufficiently late point in the search function.
    // Alpha-beta pruning isn't used when iterating over `moves` because in order to sort the moves accurately, each move's score must be fully calculated.
    let mut best_score = -isize::MAX;

    let mut scores = HashMap::new();
    for mv in moves.iter().cloned() {
        // Check for a halt command
        if let Some(halt_receiver) = halt_receiver {
            if let Ok(halt_command) = halt_receiver.try_recv() { return Err(halt_command); }
        }

        let score = -negamax(
            &board::make_move(board, mv), depth - 1, -isize::MAX, isize::MAX, halt_receiver
        )?;

        if score > best_score {
            best_score = score;
            *best_move = Some(mv.clone());
        }

        scores.insert(mv, score);
    }

    // Check for a halt command
    if let Some(halt_receiver) = halt_receiver {
        if let Ok(halt_command) = halt_receiver.try_recv() { return Err(halt_command); }
    }

    moves.sort_by_key(|mv| -scores.get(mv).unwrap());

    Ok(())
}

fn dfs_search_final(
    board: &Board, moves: &mut Vec<Move>, best_move: &mut Option<Move>, max_depth: usize, halt_receiver: Option<&mpsc::Receiver<HaltCommand>>
) -> Result<(), HaltCommand> {
    // Run depth-first search with a max depth of `depth`, utilizing alpha-beta pruning on the provided moves to maximize speed.
    let mut best_score = -isize::MAX;
    let mut alpha = -isize::MAX;

    for &mut mv in moves {
        // Check for a halt command
        if let Some(halt_receiver) = halt_receiver {
            if let Ok(halt_command) = halt_receiver.try_recv() { return Err(halt_command); }
        }

        let score = -negamax(
            &board::make_move(board, mv), max_depth - 1, -isize::MAX, -alpha, halt_receiver
        )?;

        if score > best_score {
            best_score = score;
            *best_move = Some(mv.clone());

            if score > alpha {
                alpha = score;
                if score == isize::MAX {
                    // checkmate! dubious actually...
                    return Ok(());
                }
            }
        }
    }
    Ok(())
}

fn negamax(
    board: &Board, depth: usize, mut alpha: isize, beta: isize, halt_receiver: Option<&mpsc::Receiver<HaltCommand>>
) -> Result<isize, HaltCommand> {
    // Recursively find the a position's score using [negamax](https://www.chessprogramming.org/Negamax)
    if depth == 0 {
        return Ok(relative_score(board));
    }

    let mut moves = Vec::new();
    board::gen_legal_moves(board, &mut moves);
    if moves.len() == 0 {
        return Ok(if board.is_check() {
            -isize::MAX
        } else {
            0
        });
    }

    let mut max = -isize::MAX;
    for mv in moves {
        // Check for a halt command
        if let Some(halt_receiver) = halt_receiver {
            if let Ok(halt_command) = halt_receiver.try_recv() { return Err(halt_command); }
        }

        let score = -negamax(
            &board::make_move(board, mv), depth - 1, -beta, -alpha, halt_receiver
        )?;

        if score > max {
            max = score;
            if score > alpha {
                alpha = score;
                if alpha >= beta {
                    break;
                }
            }
        }
    }
    Ok(max)
}

const MATERIAL_FACTOR: isize = 100;
const PST_FACTOR: isize = 1;

fn relative_score(board: &Board) -> isize {
    score_side(board, board.get_side_to_move()) - score_side(board, !board.get_side_to_move())
}

fn score_side(board: &Board, color: Color) -> isize {
    let mut score = 0;

    for piece in PIECES {
        let material = material(piece);
        for square in board.get_piece(piece) & board.get_color(color) {
            score += MATERIAL_FACTOR * material;
            score += PST_FACTOR * psts::get_mg(piece, color, square);
        }
    }

    score
}

const fn material(piece: Piece) -> isize {
    match piece {
        Piece::Rook => 5,
        Piece::Knight => 3,
        Piece::Bishop => 3,
        Piece::King => 0,
        Piece::Queen => 9,
        Piece::Pawn => 1
    }
}