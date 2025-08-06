mod psts;

use std::{collections::HashMap, sync::mpsc, time::Instant};

use crate::{chess::*, uci::{HaltCommand, UciGoOptions}};

const MAX_DEPTH: usize = 6;
const MAX_TIME: usize = usize::MAX; // ms

const fn next_iter_time_guess(depth: usize) -> usize {
    match depth {
        1 => 0,
        2 => 5,
        3 => 50,
        4 => 250,
        5 => 1500,
        _ => usize::MAX
    }
}

#[derive(Debug, Clone)]
pub struct SearchOptions {
    pub max_depth: usize,
    pub time: usize,
    pub search_moves: Option<Vec<Move>>,
    pub nodes: Option<usize>,
}

pub fn decide_options(board: &mut Board, go_options: UciGoOptions) -> SearchOptions {
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
        let mut total_time = 0;
        loop {
            depth += 1;
            total_time += next_iter_time_guess(depth);
            if total_time >= time {
                break;
            }
        }
        depth - 1
    };
    let max_depth = go_options.depth.unwrap_or(MAX_DEPTH).min(time_bound_depth).min(MAX_DEPTH);

    let search_moves = go_options.search_moves.map(|v| v.into_iter()
        .map(|uci| Move::from_uci(&uci, &board).unwrap())
        .collect()
    );

    let nodes = go_options.nodes;

    SearchOptions {
        max_depth,
        time,
        search_moves,
        nodes,
    }
}

pub fn search_infinite(board: &mut Board, halt_receiver: &mpsc::Receiver<HaltCommand>) -> Option<Move> {
    todo!()
}

pub fn search(board: &mut Board, options: SearchOptions, halt_receiver: &mpsc::Receiver<HaltCommand>) -> Result<Option<Move>, ()> {
    // Search for the best move in a position using [iterative deepening](https://www.chessprogramming.org/Iterative_Deepening)
    let start_time = Instant::now();

    let SearchOptions { max_depth, time, search_moves, nodes } = options;

    let mut moves = search_moves.unwrap_or(board.get_legal_moves());

    let mut best_move: Option<Move> = None;

    for depth in 1..max_depth {
        // Check for a halt command
        if let Ok(halt_cmd) = halt_receiver.try_recv() {
            match halt_cmd {
                HaltCommand::Stop => return Ok(best_move),
                HaltCommand::Quit => return Err(())
            }
        }

        // Check if we have time to do a search at this depth (TODO: do we need this after implementing stop?)
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
    if let Ok(halt_cmd) = halt_receiver.try_recv() {
        match halt_cmd {
            HaltCommand::Stop => return Ok(best_move),
            HaltCommand::Quit => return Err(())
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

fn dfs_search_and_sort(board: &mut Board, moves: &mut Vec<Move>, best_move: &mut Option<Move>, depth: usize, halt_receiver: &mpsc::Receiver<HaltCommand>) -> Result<(), HaltCommand> {
    // Run depth-first search with a max depth of `depth` and sort `moves` from worst to best.
    // The function also updates `best_move` as soon as a better move is discovered; combined with move-sorting from previous iterations,
    // this means that `best_move` will have a reasonable move at any sufficiently late point in the search function.
    // Alpha-beta pruning isn't used when iterating over `moves` because in order to sort the moves accurately, each move's score must be fully calculated.
    let mut best_score = -isize::MAX;

    let mut scores = HashMap::new();
    for mv in moves.iter().cloned() {
        // Check for a halt command
        if let Ok(halt_command) = halt_receiver.try_recv() { return Err(halt_command); }

        board.make_move(&mv, true);
        let score = -negamax(board, depth - 1, -isize::MAX, isize::MAX, halt_receiver)?;
        board.undo_move();

        if score > best_score {
            best_score = score;
            *best_move = Some(mv.clone());
        }

        scores.insert(mv, score);
    }


    // let scores: HashMap<_, _> = moves.iter().cloned().map(|mv| {
    //     board.make_move(&mv, true);
    //     let score = -negamax(board, depth - 1, -isize::MAX, isize::MAX);
    //     board.undo_move();

    //     if score > best_score {
    //         best_score = score;
    //         *best_move = Some(mv.clone());
    //     }

    //     (mv, score)
    // }).collect();

    // Check for a halt command
    if let Ok(halt_command) = halt_receiver.try_recv() { return Err(halt_command); }

    moves.sort_by_key(|mv| -scores.get(mv).unwrap());

    Ok(())
}

fn dfs_search_final(board: &mut Board, moves: &mut Vec<Move>, best_move: &mut Option<Move>, max_depth: usize, halt_receiver: &mpsc::Receiver<HaltCommand>) -> Result<(), HaltCommand> {
    // Run depth-first search with a max depth of `depth`, utilizing alpha-beta pruning on the provided moves to maximize speed.
    let mut best_score = -isize::MAX;
    let mut alpha = -isize::MAX;

    for mv in moves {
        // Check for a halt command
        if let Ok(halt_command) = halt_receiver.try_recv() { return Err(halt_command); }

        board.make_move(&mv, true);
        let score = -negamax(board, max_depth - 1, -isize::MAX, -alpha, halt_receiver)?;
        board.undo_move();

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

fn negamax(board: &mut Board, depth: usize, mut alpha: isize, beta: isize, halt_receiver: &mpsc::Receiver<HaltCommand>) -> Result<isize, HaltCommand> {
    // Recursively find the a position's score using [negamax](https://www.chessprogramming.org/Negamax)
    if depth == 0 {
        return Ok(relative_score(board));
    }

    let moves = board.get_legal_moves();
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
        if let Ok(halt_command) = halt_receiver.try_recv() { return Err(halt_command); }

        board.make_move(&mv, true);
        let score = -negamax(board, depth - 1, -beta, -alpha, halt_receiver)?;
        board.undo_move();
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

    for coord in board.find_players_pieces(color) {
        let piece = board.get_square(coord).unwrap();
        score += MATERIAL_FACTOR * material(piece.piece_type);
        score += PST_FACTOR * psts::get_mg(piece, coord);
    }

    score
}

const fn material(piece_type: PieceType) -> isize {
    match piece_type {
        PieceType::Rook => 5,
        PieceType::Knight => 3,
        PieceType::Bishop => 3,
        PieceType::King => 0,
        PieceType::Queen => 9,
        PieceType::Pawn => 1
    }
}