use crate::board::Board;
use crate::fen::STARTPOS_FEN;
use crate::r#move::Move;
use crate::move_gen::generate_moves;
use crate::search;

// Public entrypoint for the UCI protocol loop.
// Keeps debug logging on stderr; UCI-required outputs on stdout.
pub fn run() {
    use std::io::{self, BufRead, Write};

    eprintln!("[debug] Engine starting. Entering UCI loop...");

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    let mut engine = UciEngine::new();

    // Print nothing until GUI says `uci`, but we do keep debug logs on stderr.
    for line in stdin.lock().lines() {
        let Ok(raw) = line else { break };
        let cmd = raw.trim();
        if cmd.is_empty() {
            continue;
        }
        eprintln!("[debug] <- {}", cmd);

        let keep_running = engine.handle_line(cmd);
        let _ = stdout.flush();
        if !keep_running {
            break;
        }
    }
}

// ------------------------- Helpers -------------------------

fn startpos_board() -> Board {
    Board::from_fen(STARTPOS_FEN)
}

// ------------------------- UCI Engine -------------------------

struct UciEngine {
    board: Board,
    default_depth: u8,
}

impl UciEngine {
    fn new() -> Self {
        Self {
            board: startpos_board(),
            default_depth: 3,
        }
    }

    /// Handle a single input line. Returns false if engine should quit.
    fn handle_line(&mut self, cmd: &str) -> bool {
        let mut tokens = cmd.split_whitespace().peekable();
        let head = tokens.next().unwrap_or("");
        match head {
            // --- Handshake ---
            "uci" => self.handle_uci(),
            "isready" => self.handle_isready(),
            "ucinewgame" => self.handle_ucinewgame(),
            // --- Options (ignored for now) ---
            "setoption" => self.handle_setoption(cmd),
            "register" => self.handle_register(cmd),
            // --- Position ---
            "position" => self.handle_position(tokens),
            // --- Go / Stop ---
            "go" => self.handle_go(tokens),
            "stop" => self.handle_stop(),
            // --- Custom helper to adjust defaults ---
            "setdepth" => self.handle_setdepth(tokens),
            // --- Quit ---
            "quit" => return self.handle_quit(),
            other => {
                eprintln!("[debug] unknown/unsupported command: '{}'", other);
            }
        }
        true
    }

    fn handle_uci(&mut self) {
        println!("id name 2025-11-chess");
        println!("id author Christoph");
        // Options could be printed here later.
        println!("uciok");
    }

    fn handle_isready(&mut self) {
        println!("readyok");
    }

    fn handle_ucinewgame(&mut self) {
        self.board = startpos_board();
        eprintln!("[debug] ucinewgame: state cleared");
    }

    fn handle_setoption(&mut self, full_cmd: &str) {
        eprintln!("[debug] setoption (ignored): {}", full_cmd);
    }
    fn handle_register(&mut self, full_cmd: &str) {
        eprintln!("[debug] register (ignored): {}", full_cmd);
    }

    fn handle_position<'a>(&mut self, tokens: impl Iterator<Item = &'a str>) {
        match apply_position_command(&mut self.board, tokens) {
            Ok(()) => eprintln!("[debug] position applied: {}", self.board.to_fen()),
            Err(e) => eprintln!("[debug] position error: {}", e),
        }
    }

    fn handle_go<'a>(&mut self, tokens: impl Iterator<Item = &'a str>) {
        let mut depth: Option<u8> = None;
        let mut t = tokens.peekable();
        while let Some(tok) = t.next() {
            match tok {
                "depth" => {
                    if let Some(n) = t.next() {
                        if let Ok(v) = n.parse::<u8>() {
                            depth = Some(v);
                        } else {
                            eprintln!("[debug] go depth: invalid number '{}'", n);
                        }
                    }
                }
                // Explicitly unsupported (ignored but logged)
                "wtime" | "btime" | "winc" | "binc" | "movestogo" | "movetime" | "infinite"
                | "ponder" => {
                    if matches!(
                        tok,
                        "wtime" | "btime" | "winc" | "binc" | "movestogo" | "movetime"
                    ) {
                        let _ = t.next();
                    }
                    eprintln!("[debug] go: ignoring token '{}' for now", tok);
                }
                other => eprintln!("[debug] go: unknown token '{}' (ignored)", other),
            }
        }

        let d = depth.unwrap_or(self.default_depth);
        let d = 4;
        /// TODO: use real depth, limited for now to keep responsive...
        eprintln!("[debug] go: starting search at depth {}", d);

        // Synchronous search, no stop support yet (planned for future threading)
        let mut b = self.board; // copy
        let result = search::find_best_move(&mut b, d);

        println!("info depth {} score cp {}", d, result.score);
        if let Some(m) = result.move_ {
            println!("bestmove {}", uci_move_string(m));
        } else {
            println!("bestmove 0000");
        }
    }

    fn handle_stop(&mut self) {
        eprintln!("[debug] stop: no active async search; ignoring");
    }

    fn handle_setdepth<'a>(&mut self, mut tokens: impl Iterator<Item = &'a str>) {
        if let Some(n) = tokens.next() {
            if let Ok(v) = n.parse::<u8>() {
                self.default_depth = v;
            }
        }
        eprintln!("[debug] setdepth: default_depth={}", self.default_depth);
    }

    fn handle_quit(&mut self) -> bool {
        eprintln!("[debug] quit: exiting UCI loop");
        false
    }
}

fn apply_position_command<'a>(
    board: &mut Board,
    mut tokens: impl Iterator<Item = &'a str>,
) -> Result<(), String> {
    // Grammar we handle:
    // position startpos [moves m1 m2 ...]
    // position fen <FEN-6-fields> [moves m1 m2 ...]

    let first = tokens.next().ok_or("position: missing argument")?;
    match first {
        "startpos" => {
            *board = startpos_board();
        }
        "fen" => {
            // Collect exactly 6 fields of FEN
            let f0 = tokens.next().ok_or("position fen: missing field 1")?;
            let f1 = tokens.next().ok_or("position fen: missing field 2")?;
            let f2 = tokens.next().ok_or("position fen: missing field 3")?;
            let f3 = tokens.next().ok_or("position fen: missing field 4")?;
            let f4 = tokens.next().ok_or("position fen: missing field 5")?;
            let f5 = tokens.next().ok_or("position fen: missing field 6")?;
            let fen = format!("{} {} {} {} {} {}", f0, f1, f2, f3, f4, f5);
            *board = Board::from_fen(&fen);
        }
        other => {
            return Err(format!(
                "position: expected 'startpos' or 'fen', got '{}'",
                other
            ));
        }
    }

    // Optional moves
    if let Some(next) = tokens.next() {
        if next == "moves" {
            for mtoken in tokens {
                apply_one_move_token(board, mtoken)?;
            }
        }
    }

    Ok(())
}

fn apply_one_move_token(board: &mut Board, token: &str) -> Result<(), String> {
    // Generate legal moves and find the one that matches the UCI token.
    let moves = generate_moves(board);
    for m in moves {
        if token_matches_uci_move(token, m) {
            board.make_move(m);
            return Ok(());
        }
    }
    Err(format!(
        "no legal move matching token '{}' from position {}",
        token,
        board.to_fen()
    ))
}

fn token_matches_uci_move(token: &str, m: Move) -> bool {
    // Match either our Display form (e.g., e7e8=Q) or pure UCI (e7e8q)
    let display_form = m.to_string();
    if token.eq_ignore_ascii_case(&display_form) {
        return true;
    }
    // Convert display form to UCI compact for compare: drop '=', lowercase promo
    let compact = display_to_compact_uci(&display_form);
    token.eq_ignore_ascii_case(&compact)
}

fn display_to_compact_uci(s: &str) -> String {
    // e2e4 -> e2e4
    // e7e8=Q -> e7e8q
    if let Some(eq_pos) = s.find('=') {
        let mut out = String::with_capacity(s.len());
        out.push_str(&s[..eq_pos]);
        if let Some(ch) = s.chars().nth(eq_pos + 1) {
            out.push(ch.to_ascii_lowercase());
        }
        out
    } else {
        s.to_string()
    }
}

fn uci_move_string(m: Move) -> String {
    // Convert our Display form (e.g., e7e8=Q) into pure UCI (e7e8q)
    display_to_compact_uci(&m.to_string())
}
