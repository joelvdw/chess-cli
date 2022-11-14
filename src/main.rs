mod chess;
use std::io;
use io::Write;

use chess::{Board, Move, Color, GameState, MoveOk, Symbol};
use chess::MoveErr;

enum PlayerInput {
    Move(Move),
    Draw,
    Surrender
}

fn print_moveerr(err: MoveErr) {
    match err {
        MoveErr::BadColor => println!("You cannot move this piece"),
        MoveErr::EmptyCase => println!("You chose ab empty case"),
        MoveErr::InvalidInput => println!("Invalid input. Valid example: f7f6"),
        MoveErr::InvalidMove => println!("This piace cannot do that"),
        MoveErr::KingCheck => println!("King cannot move into check position"),
        MoveErr::MoveOnCheck => println!("You are on check, you must end it"),
        MoveErr::OutOfBound => println!("Out of bound move")
    }
}

fn ask_move() -> PlayerInput {
    let mut input: Option<PlayerInput> = None;
    while input.is_none() {
        print!("> ");
        io::stdout().flush().expect("Flush failed");

        let mut str = String::new();
        match io::stdin().read_line(&mut str) {
            Err(e) => panic!("{e}"),
            Ok(_) => {
                match str.to_lowercase().as_str() {
                    "draw" => input = Some(PlayerInput::Draw),
                    "surrend" | "surrender" => input = Some(PlayerInput::Surrender),
                    v => {
                        match Move::from_str(v) {
                            Ok(m) => input = Some(PlayerInput::Move(m)),
                            Err(e) => print_moveerr(e)
                        }
                    }
                }
            }
        }
    }

    input.unwrap()
}

fn ask_draw() -> bool {
    println!("The other player asked for a draw. Do you accept it ?");
    let mut str = String::new();
    match io::stdin().read_line(&mut str) {
        Err(e) => panic!("{e}"),
        Ok(_) => {
            matches!(str.to_lowercase().as_str(), "yes" | "y" | "ok")
        }
    }
}

fn ask_promote() -> Symbol {
    println!("Your pawn can be promoted. Which symbol do you want ?");
    println!(" [Q] Queen");
    println!(" [r] Rook");
    println!(" [n] Knight");
    println!(" [b] Bishop");
    let mut str = String::new();
    match io::stdin().read_line(&mut str) {
        Err(e) => panic!("{e}"),
        Ok(_) => {
            match str.to_lowercase().as_str() {
                "Q" | "queen" => Symbol::Queen,
                "r" | "rook" => Symbol::Rook,
                "n" | "knight" => Symbol::Knight,
                "b" | "bishop" => Symbol::Bishop,
                _ => Symbol::Queen
            }
        }
    }
}

fn game_loop(mut board: Board) {
    let mut turn = Color::White;
    let mut draw_asked = false;

    loop {
        board.print(turn == Color::Black);
        if draw_asked {
            if ask_draw() {
                board.set_gamestate(GameState::Draw);
                break;
            }
            draw_asked = false;
        } else {
            match ask_move() {
                PlayerInput::Move(m) => {
                    match board.apply(
                        m, 
                        turn
                    ) {
                        Ok(mo) => {
                            if let MoveOk::Promote(x, y) = mo {
                                let s = ask_promote();
                                board.promote(x, y, s)
                            }
                            turn = turn.invert();
                        },
                        Err(e) => print_moveerr(e)
                    }
                },
                PlayerInput::Draw => {
                    draw_asked = true;
                    turn = turn.invert();
                },
                PlayerInput::Surrender => {
                    board.set_gamestate(GameState::Surrender(turn));
                    break;
                }
            }
        }

        match board.check_state {
            GameState::Check(p) => {
                println!("{} is on check", p);
            },
            GameState::Checkmate(p) => {
                println!("{} won by checkmate !", p.invert());
                break;
            },
            GameState::Surrender(p) => {
                println!("{} has surrender. {} won !", p, p.invert());
                break;
            },
            GameState::Draw => {
                println!("The game is a draw !");
                break;
            },
            GameState::No => (),
        }
    }

    // for m in ["b1a3", "e7e5", "d2d3", "b7b5", "c1f4", "g8f6", "e2e3", "e5f4", "e3f4", "h7h6", "d1h5", "f8a3", "e1c1", "e8g8", "h5h6", "a3b2", "h6h8"] {
    //     println!("{}: {:?}", m, board.apply(Move::from_str(m).unwrap(), if white_turn { Color::White } else { Color::Black }));
    //     board.print(white_turn);
    //     println!("{:?}", board.check_state);
    //     white_turn = !white_turn;
    // }
}

fn main() {
    let b = Board::new();
    println!("Welcome in the Chess CLI game !");
    println!("White pieces begin.");
    println!("To play a move, enter the start and end case, i.e. f6f5 to move the piece on f6 to f5.");
    game_loop(b);
}
