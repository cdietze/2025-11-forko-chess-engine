# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a chess engine written in Rust that implements the UCI (Universal Chess Interface) protocol. The engine uses bitboard-based move generation, alpha-beta pruning search, and PeSTO's evaluation function.

## Development Commands

### Build and Run
```bash
cargo build           # Build the project
cargo run             # Run the engine (enters UCI mode)
cargo build --release # Build optimized version
```

### Testing
```bash
cargo test                    # Run all tests
cargo test <test_name>        # Run specific test
cargo test -- --nocapture     # Run tests with output
cargo test --relase           # Run tests in release mode, which will include some perft testsafa
```

Note: Some tests like `test_self_play` are marked with `#[ignore]` due to long runtime. Run them explicitly with:
```bash
cargo test test_self_play -- --ignored
```

### Formatting and Linting
```bash
cargo fmt        # Format code
cargo fmt --all  # Format all packages
cargo clippy     # Run linter
```

### Pre-commit Hook
Install the pre-commit hook to automatically run `cargo fmt` and `cargo test` before commits:
```bash
ln -sf ../../scripts/hooks/pre-commit .git/hooks/pre-commit
```

## Architecture

### Core Representation

**BitBoard** (`src/bitboard.rs`, `src/bitboard_ops.rs`):
- The engine uses a bitboard representation where each piece type and color is represented by a 64-bit unsigned integer
- Each bit corresponds to a square on the chessboard (a1=0, h8=63)
- Bitboard operations are implemented as const methods for performance
- Key operations: `and`, `or`, `xor`, `not`, `shl`, `shr`, `bit_scan_forward`, `bit_scan_backward`

**Board** (`src/board.rs`):
- The board state is represented by:
  - `white: BitBoard` - all white pieces
  - `pieces: [BitBoard; 6]` - one bitboard per piece type (King, Queen, Rook, Bishop, Knight, Pawn)
  - `white_to_move: bool` - side to move
  - `en_passant: Square` - en passant target square
  - `castling_rights: [CastlingRights; 2]` - castling rights per color
- Board is `Copy` for efficient cloning during search

**Square** (`src/square.rs`):
- Squares are represented as `Square(u8)` where 0=a1, 63=h8
- Square indexing: `rank * 8 + file`

### Move Generation

**Move Generation** (`src/move_gen.rs`):
- Generates pseudo-legal moves and filters them for legality
- Handles checks, pins, and x-ray attacks
- Key functions:
  - `generate_moves()` - generates all pseudo-legal moves
  - `is_legal()` - validates if a move is legal
  - Pin detection uses precomputed ray attacks
  - Check detection considers all opponent piece attacks to the king

**Precomputed Tables** (`src/precomputed.rs`):
- Attack tables and rays are precomputed at compile time using const functions
- `RAYS[64][8]` - ray attacks in 8 directions for each square
- `KING_MOVES[64]` - king move bitboards
- `KNIGHT_ATTACKS[64]` - knight attack bitboards
- `LINE_BB[64][64]` - complete lines between squares
- `BETWEEN_BB[64][64]` - squares strictly between two squares
- Castling configurations stored in `CASTLING_SETUPS`

### Search and Evaluation

**Search** (`src/search.rs`):
- Uses negamax with alpha-beta pruning
- Function signature: `nega_max_impl(board, depth, alpha, beta, info, track_move)`
- Currently clones boards during search (TODO: implement unmake_move for efficiency)
- Returns `SearchResult` with score and best move
- Detects checkmate (scored as `CHECKMATE_SCORE - depth`) and stalemate

**Evaluation** (`src/eval.rs`):
- Implements PeSTO's evaluation function (Piece-Square Tables Only)
- Uses tapered eval interpolating between midgame and endgame based on material
- Piece-square tables (PSTs) are mirrored for black pieces using `sq ^ 56`
- Returns score from white's perspective

### UCI Interface

**UCI** (`src/uci.rs`):
- Implements the Universal Chess Interface protocol
- Main entrypoint: `uci::run()` - enters the UCI command loop
- Uses stdin/stdout for protocol communication, stderr for debug logging

### Other Modules

**FEN** (`src/fen.rs`):
- Parses FEN (Forsyth-Edwards Notation) strings into `Board` structs

**Perft** (`src/perft.rs`):
- Performance testing for move generation correctness
- Tests validate against known perft results from chessprogramming.org

**Geometry** (`src/geometry.rs`):
- Defines `Dir4` (Rank, File, Diagonal, AntiDiagonal) and `Dir8` (8 compass directions)
- Helper functions for geometric calculations on the board

**Build Script** (`build.rs`):
- Captures git tag and commit hash at build time
- Sets environment variables `BUILD_GIT_TAG` and `BUILD_GIT_COMMIT`

## Key Implementation Details

### Coordinate System
- Squares are indexed with a1=0, h1=7, a8=56, h8=63
- Ranks go from 0 (first rank) to 7 (eighth rank)
- Files go from 0 (a-file) to 7 (h-file)

### Move Representation
Move encoding uses a 16-bit format capturing from/to squares, piece types, and special flags (promotion, castling, en passant).

### Pinned Pieces
The engine uses x-ray attacks to detect pinned pieces. A piece is pinned if moving it would expose the king to check along a rank, file, or diagonal.
