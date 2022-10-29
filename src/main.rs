#![allow(dead_code)]
#![allow(unused_variables)]
mod chess;
use std::io;
use std::io::prelude::*;

use chess::{Board, Move, Color};

fn game_loop(mut board: Board) {
    let mut white_turn = true;

    for m in ["b1a3", "e7e5", "d2d3", "b7b5", "c1f4", "g8f6", "e2e3", "e5f4", "e3f4", "h7h6", "d1h5", "f8a3", "e1c1", "e8g8"] {
        println!("{}: {:?}", m, board.apply(Move::from_str(m).unwrap(), if white_turn { Color::White } else { Color::Black }));
        board.print(!white_turn);
        white_turn = !white_turn;
        //let _ = io::stdin().read(&mut [0u8]).unwrap();
    }

    // TODO: can ask for a draw, and an ok from other player
}

fn main() {
    let b = Board::new();
    b.print(false);
    game_loop(b);
}
