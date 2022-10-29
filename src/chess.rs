use std::fmt;

#[derive(Clone, Copy, PartialEq, Debug)]
enum Symbol {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King
}
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Color {
    White,
    Black
}
#[derive(Clone, Debug)]
struct Piece {
    symbol: Symbol,
    color: Color,
    has_moved: bool
}

type Case = Option<Piece>;

#[derive(Debug)]
pub enum MoveErr {
    InvalidInput,
    OutOfBound,
    EmptyCase,
    BadColor,
    InvalidMove,
    KingCheck
}
#[derive(Debug)]
pub enum MoveOk {
    Move,
    Promote,
    Capture(usize, usize),
    Castling { king: (usize, usize), rook: (usize, usize) }
}

#[derive(Copy, Clone, PartialEq)]
pub enum GameState {
    Check(Color), // Contains the player color who did the check
    Checkmate(Color), // Contains the winner player color
    Draw,
    No
}

#[derive(Clone, Copy, Debug)]
pub struct Move {
    from: (usize, usize),
    to: (usize, usize)
}

#[derive(Clone)]
pub struct Board {
    data: [[Case; 8]; 8], // The board always must be square
    check_state: GameState,
    last_move: Option<Move>
}

impl Color {
    pub fn invert(self) -> Self {
      if self == Color::White { Color::Black } else { Color::White }
    }
}

impl Move {
    fn is_out(from: (char, char), to: (char, char)) -> bool {
        from.0 < 'a' || from.0 > 'h' || from.1 < '1' || from.1 > '8' ||
        to.0 < 'a' || to.0 > 'h' || to.1 < '1' || to.1 > '8'
    }

    pub fn from_str(input: &str) -> Result<Move, MoveErr> {
        let clean: Vec<char> = input.to_ascii_lowercase().chars().filter(|c| c.is_alphanumeric()).collect();
        if clean.len() != 4 {
            return Err(MoveErr::InvalidInput)
        }

        let (from, to) = ((clean[0], clean[1]), (clean[2], clean[3]));
        if Move::is_out(from, to) {
            Err(MoveErr::OutOfBound)
        } else {
            let y0 = from.0 as usize - 'a' as usize;
            let y1 = to.0 as usize - 'a' as usize;
            let x0 = 8 - from.1.to_digit(10).unwrap() as usize;
            let x1 = 8 - to.1.to_digit(10).unwrap() as usize;
            Ok(Move { from: (x0, y0), to: (x1, y1) })
        }
    }
}

// Board constructors
impl Board {
    #[allow(dead_code)]
    pub fn empty() -> Board {
        let data = [(); 8].map(|_| [(); 8].map(|_| None));
        Board { data, check_state: GameState::No, last_move: None }
    }

    pub fn new() -> Board {
        let symbols = [Symbol::Rook, Symbol::Knight, Symbol::Bishop, Symbol::Queen, Symbol::King, Symbol::Bishop, Symbol::Knight, Symbol::Rook];
        let data = [
            symbols.map(|s| Some(Piece { symbol: s, color: Color::Black, has_moved: false })),
            [(); 8].map(|_| Some(Piece { symbol: Symbol::Pawn, color: Color::Black, has_moved: false })),
            [(); 8].map(|_| None),
            [(); 8].map(|_| None),
            [(); 8].map(|_| None),
            [(); 8].map(|_| None),
            [(); 8].map(|_| Some(Piece { symbol: Symbol::Pawn, color: Color::White, has_moved: false })),
            symbols.map(|s| Some(Piece { symbol: s, color: Color::White, has_moved: false }))
        ];
        Board { data, check_state: GameState::No, last_move: None }
    }
}
// Board methods
impl Board {
    /**
     * Test if a move is a valid "en passant" pawn move on the current board
     */
    fn is_en_passant(&self, movement: Move) -> Result<MoveOk, MoveErr> {
        if let Some(m) = self.last_move {
            if m.to.0 == movement.from.0 && m.to.1 == movement.to.1 
            && (m.from.0 + 2) == m.to.0 && m.from.1 == m.to.1 {
                let c_passant = &self.data[movement.from.0][movement.to.1];
                match c_passant {
                    Some(Piece { symbol: Symbol::Pawn, .. }) => Ok(MoveOk::Capture(movement.from.0, movement.to.1)),
                    _ => Err(MoveErr::InvalidMove)
                }
            } else {
                Err(MoveErr::InvalidMove)
            }
        } else {
            Err(MoveErr::InvalidMove)
        }
    }

    /**
     * Test if a move is a valid pawn move on the current board
     */
    fn is_pawn_move(&self, piece: &Piece, movement: Move, dx: i32, dy: i32, c_to: &Case) -> Result<MoveOk, MoveErr> {
        let (dir, end) = 
            match piece.color { 
                Color::White => (-1, 0),
                Color::Black => (1, 7)
            };

        if dx == dir && dy == 0 && c_to.is_none() {
            // Basic move
            if movement.to.0 == end {
                Ok(MoveOk::Promote)
            } else {
                Ok(MoveOk::Move)
            }
        } else if dx == (2*dir) && dy == 0 && !piece.has_moved && c_to.is_none()
                && self.data[(movement.from.0 as i32 + dir) as usize][movement.from.1].is_none() {
            // First 2-case move
            Ok(MoveOk::Move)
        } else if dx == dir && (dy == 1 || dy == -1) {
            // Pawn capture
            match c_to {
                Some(Piece { color: c, .. }) if c != &piece.color => Ok(MoveOk::Capture(movement.to.0, movement.to.1)),
                None => {
                    self.is_en_passant(movement)
                },
                _ => Err(MoveErr::InvalidMove)
            }
        } else {
            Err(MoveErr::InvalidMove)
        }
    }

    /**
     * Test if a move is a valid castling move on the currnet board
     */
    fn is_castling(&self, piece: &Piece, movement: Move, dy: i32) -> Result<MoveOk, MoveErr> {
        if piece.has_moved {
            return Err(MoveErr::InvalidMove);
        }

        let (rook, rook_move) = if dy > 0 { (7, -2i32) } else { (0, 3) };
        if let Some(rook_p) = &self.data[movement.from.0][rook] {

            if rook_p.has_moved {
                return Err(MoveErr::InvalidMove);
            }

            // Cases between must be free
            for i in 1..=rook_move.abs() {
                let y = if rook_move < 0 { rook - i as usize } else { rook + i as usize };
                if self.data[movement.from.0][y].is_some() {
                    return Err(MoveErr::InvalidMove);
                }
            }

            let valid = MoveOk::Castling { king: movement.from, rook: (movement.from.0, rook) };
            // King must not be in check in transition or final position
            let trans_move = Move { from: movement.from, to: (movement.from.0, (movement.from.1 as i32 + (dy/2)) as usize) };
            if self.is_move_check(trans_move, &MoveOk::Move, piece.color.invert())
                    || self.is_move_check(movement, &valid, piece.color.invert()) {
                return Err(MoveErr::InvalidMove)
            }
            
            Ok(valid)
        } else {
            Err(MoveErr::InvalidMove)
        }
    }
    
    /**
     * Test if a move is a valid king move on the current board
     */
    fn is_king_move(&self, piece: &Piece, movement: Move, dx: i32, dy: i32, c_to: &Case) -> Result<MoveOk, MoveErr> {
        if dx == 0 && dy.abs() == 2 {
            self.is_castling(piece, movement, dy)
        } else if (dx == 0 || dx.abs() == 1) && (dy == 0 || dy.abs() == 1) {
            let res = match c_to {
                None => Ok(MoveOk::Move),
                Some(Piece { color: c, .. }) if c != &piece.color => Ok(MoveOk::Capture(movement.to.0, movement.to.1)),
                _ => Err(MoveErr::InvalidMove)
            };

            // A king cannot move into check position
            if let Ok(move_type) = res.as_ref() {
                if self.is_move_check(movement, move_type, piece.color.invert()) {
                    Err(MoveErr::KingCheck)
                } else {
                    res
                }
            } else {
                res
            }
        } else {
            Err(MoveErr::InvalidMove)
        }
    }

    /**
     * Test if a linear (horizontal, vertical, diagonal) move can be made on the current board
     */
    fn is_linear_move(&self, movement: Move, color: Color, dx: i32, dy: i32, c_to: &Case) -> Result<MoveOk, MoveErr> {
        // Test if the way if free
        for i in 1..(dx.abs().max(dy.abs())) {
            let ix = if dx < 0 { -i } else if dx > 0 { i } else { 0 };
            let iy = if dy < 0 { -i } else if dy > 0 { i } else { 0 };
            if self.data[(movement.from.0 as i32 + ix) as usize][(movement.from.1 as i32 + iy) as usize].is_some() {
                return Err(MoveErr::InvalidMove);
            }
        }
        
        match c_to {
            None => Ok(MoveOk::Move),
            Some(Piece { color: c, .. }) if *c != color => Ok(MoveOk::Capture(movement.to.0, movement.to.1)),
            _ => Err(MoveErr::InvalidMove)
        }
    }

    /**
     * Test if a move is valid on the current board
     */
    fn is_move_valid(&self, movement: Move, player: Color) -> Result<MoveOk, MoveErr> {
        use Symbol::*;
        let (dx, dy) = (
            movement.to.0 as i32 - movement.from.0 as i32,
            movement.to.1 as i32 - movement.from.1 as i32
        );
        let c_from = &self.data[movement.from.0][movement.from.1];
        let c_to = &self.data[movement.to.0][movement.to.1];

        if let Some(p) = c_from { 
            if player != p.color {
                return Err(MoveErr::BadColor);
            }
        }
        if dx == 0 && dy == 0 {
            return Err(MoveErr::InvalidMove);
        }
       
        match c_from {
            Some(p) if p.symbol == Pawn => {
                self.is_pawn_move(p, movement, dx, dy, c_to)
            },
            Some(Piece { symbol: Rook, color, .. }) => {
                if (dx == 0 && dy != 0) || (dx != 0 && dy == 0) {
                    self.is_linear_move(movement, *color, dx, dy, c_to)
                } else {
                    Err(MoveErr::InvalidMove)
                }
            },
            Some(Piece { symbol: Knight, color, .. }) => {
                if (dx.abs() == 1 && dy.abs() == 2) || (dx.abs() == 2 && dy.abs() == 1) {
                    match c_to {
                        None => Ok(MoveOk::Move),
                        Some(Piece { color: c, .. }) if c != color => Ok(MoveOk::Capture(movement.to.0, movement.to.1)),
                        _ => Err(MoveErr::InvalidMove)
                    }
                } else {
                    Err(MoveErr::InvalidMove)
                }
            },
            Some(Piece { symbol: Bishop, color, .. }) => {
                if dx.abs() == dy.abs() {
                    self.is_linear_move(movement, *color, dx, dy, c_to)
                } else {
                    Err(MoveErr::InvalidMove)
                }
            },
            Some(Piece { symbol: Queen, color, .. }) => {
                if (dx.abs() == dy.abs()) || (dx == 0 && dy != 0) || (dx != 0 && dy == 0) {
                    self.is_linear_move(movement, *color, dx, dy, c_to)
                } else {
                    Err(MoveErr::InvalidMove)
                }
            },
            Some(p) if p.symbol == King => {
                self.is_king_move(p, movement, dx, dy, c_to)
            },
            _ => Err(MoveErr::EmptyCase)
        }
    }

    /**
     * Test if a move would create a check on next turn
     * `player` is the player that could do a check with the move
     */
    fn is_move_check(&self, movement: Move, move_type: &MoveOk, player: Color) -> bool {
        let mut b = self.clone();
        b.do_move(movement, move_type, player);
        
        b.check_state != GameState::No
    }

    /**
     * Test if coordinates are inside the board
     */
    fn is_inside(&self, i: i32, j: i32) -> bool {
        0 < i && i < self.data.len() as i32 && 0 < j && j < self.data[0].len() as i32
    }

    /**
     * Get all possible moves for a piece in the given position
     * Does not take into account the state of the board and other pieces
     */
    fn possible_moves(&self, piece: &Piece, i: usize, j: usize) -> Vec<Move> {
        use Symbol::*;
        let mut ids: Vec<(i32, i32)> = Vec::with_capacity(64);
        match piece {
            Piece { symbol: Pawn, color, has_moved } => {
                let dir = if *color == Color::White { -1 } else { 1 };
                ids.extend([(dir, 0), (dir, -1), (dir, 1)]);
                if !has_moved {
                    ids.push((2*dir, 0));
                }
            },
            Piece { symbol: Knight, .. } => {
                for m in [-2, 2] {
                    for n in [-1, 1] {
                        ids.push((m, n));
                        ids.push((n, m));
                    }
                }
            },
            Piece { symbol: Rook, .. } => {
                for m in 1..(self.data.len() as i32) {
                    ids.extend([(-m, 0), (m, 0), (0, -m), (0, m)]);
                }
            },
            Piece { symbol: Bishop, .. } => {
                for m in 1..(self.data.len() as i32) {
                    ids.extend([(-m, -m), (m, -m), (m, -m), (m, m)]);
                }
            },
            Piece { symbol: Queen, .. } => {
                for m in 1..(self.data.len() as i32) {
                    ids.extend([(-m, 0), (m, 0), (0, -m), (0, m)]);
                    ids.extend([(-m, -m), (m, -m), (m, -m), (m, m)]);
                }
            },
            Piece { symbol: King, .. } => {
                for m in [-1, 0, 1] {
                    for n in [-1, 0, 1] {
                        if m != 0 || n != 0 {
                            ids.push((m, n));
                        }
                    }
                }
                // Castling
                if !piece.has_moved {
                    ids.extend([(0, -2), (0, 2)]);
                }
            }
        }

        // Convert deplacement into absolute indices and verify its inside the board
        ids.iter()
        .filter_map(|(dx, dy)| {
            let x = i as i32 + dx;
            let y = j as i32 + dy;
            self.is_inside(x, y);
            Some((x as usize, y as usize))
        })
        .map(|(x, y)| Move { from: (i, j), to: (x, y) })
        .collect()
    }

    /**
     * Calculate all valid moves for the player on the current board
     */
    pub fn all_moves(&self, player: Color) -> Vec<(Move, MoveOk)> {
        let mut moves = Vec::with_capacity(50);
        for i in 0..self.data.len() {
            for j in 0..self.data[0].len() {
                match &self.data[i][j] {
                    Some(piece) if piece.color == player => 
                        moves.extend(self.possible_moves(piece, i, j).iter().filter_map(|m| {
                            match self.is_move_valid(*m, player) {
                                Ok(t) => Some((*m, t)),
                                Err(_) => None
                            }
                        })),
                    _ => {}
                }
            }
        }

        println!("Moves len: {}", moves.len());
        moves
    }

    fn is_draw(&self, next_player: Color) -> bool {
        if self.check_state == GameState::Draw {
            return true;
        }
        // Stalemate (no legal move and not in check)
        if self.check_state == GameState::No && self.all_moves(next_player).len() == 0 {
            return true;
        }

        false
    }

    /**
     * Test if a player is in check
     * `player` is the player that could do a check
     */
    fn is_check(&self, player: Color) -> GameState {
        let check_moves = self.all_moves(player);
        let check = check_moves.iter().any(|x| self.is_move_check(x.0, &x.1, player));

        if check {
            let mate = self.all_moves(player.invert()).iter().all(|x| self.is_move_check(x.0, &x.1, player));
            if mate {
                GameState::Checkmate(player)
            } else {
                GameState::Check(player)
            }
        } else {
            GameState::No
        }
    }

    fn set_move(&mut self, case: (usize, usize)) {
        let case = &mut self.data[case.0][case.1];
        match case {
            Some(p) => p.has_moved = true,
            _ => ()           
        }
    }

    /**
     * Actually apply a move on the board
     * `player` is the player that could do a check with the move
     */
    fn do_move(&mut self, movement: Move, move_type: &MoveOk, player: Color) {
        match move_type {
            MoveOk::Move |
            MoveOk::Promote => {
                self.data[movement.to.0][movement.to.1] = self.data[movement.from.0][movement.from.1].take();
                self.set_move(movement.to);
            },
            MoveOk::Capture(x, y) => {
                self.data[*x][*y] = None;
                self.data[movement.to.0][movement.to.1] = self.data[movement.from.0][movement.from.1].take();
                self.set_move(movement.to);
            },
            MoveOk::Castling { king, rook } => {
                if king.1 < rook.1 {
                    self.data[king.0][king.1+2] = self.data[king.0][king.1].take();
                    self.data[rook.0][rook.1-2] = self.data[rook.0][rook.1].take();
                    self.set_move((king.0, king.1+2));
                    self.set_move((rook.0, rook.1-2));
                } else {
                    self.data[king.0][king.1-2] = self.data[king.0][king.1].take();
                    self.data[rook.0][rook.1+3] = self.data[rook.0][rook.1].take();
                    self.set_move((king.0, king.1-2));
                    self.set_move((rook.0, rook.1+3));
                }
            }
        }

        self.check_state = self.is_check(player);
        if self.is_draw(player.invert()) {
            self.check_state = GameState::Draw;
        }
        self.last_move = Some(movement);
    }

    pub fn apply(&mut self, movement: Move, player: Color) -> Result<MoveOk, MoveErr> {
        let res = self.is_move_valid(movement, player)?;
        self.do_move(movement, &res, player);

        Ok(res)
    }
}
// Board print methods
impl Board {
    pub fn print(&self, invert: bool) {
        print!("{}", self.to_string(invert));
    }

    fn to_string(&self, invert: bool) -> String {
        let mut out = String::new();

        let iter: Vec<usize> = if invert { (0..self.data.len()).rev().collect() } else { (0..self.data.len()).collect() };
        for i in iter {
            out.push_str(&format!("{} ", 8-i));
            for p in &self.data[i] {
                let c = match p {
                    Some(Piece { symbol: Symbol::Pawn, color: Color::Black, .. }) => '♙',
                    Some(Piece { symbol: Symbol::Knight, color: Color::Black, .. }) => '♘',
                    Some(Piece { symbol: Symbol::Bishop, color: Color::Black, .. }) => '♗',
                    Some(Piece { symbol: Symbol::Rook, color: Color::Black, .. }) => '♖',
                    Some(Piece { symbol: Symbol::Queen, color: Color::Black, .. }) => '♕',
                    Some(Piece { symbol: Symbol::King, color: Color::Black, .. }) => '♔',

                    Some(Piece { symbol: Symbol::Pawn, color: Color::White, .. }) => '♟',
                    Some(Piece { symbol: Symbol::Knight, color: Color::White, .. }) => '♞',
                    Some(Piece { symbol: Symbol::Bishop, color: Color::White, .. }) => '♝',
                    Some(Piece { symbol: Symbol::Rook, color: Color::White, .. }) => '♜',
                    Some(Piece { symbol: Symbol::Queen, color: Color::White, .. }) => '♛',
                    Some(Piece { symbol: Symbol::King, color: Color::White, .. }) => '♚',
                    None => ' '
                };
                out.push_str(&format!("{c} "));
            }
            out.push_str(&format!("\n"));
        }
        out.push_str(&format!("  A B C D E F G H\n"));
        out
    }
}
impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string(false))?;
        Ok(())
    }
}
