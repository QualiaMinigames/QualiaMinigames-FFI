// Copyright (C) 2026 QualiaMinigames contributors
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License as published by the Free
// Software Foundation; either version 3 of the License, or (at your option) any
// later version.
//
// This library is distributed in the hope that it will be useful, but WITHOUT
// ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
// FOR A PARTICULAR PURPOSE. See the GNU Lesser General Public License for more
// details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with this library; if not, see <https://www.gnu.org/licenses/>.

//! UniFFI bindings for the QualiaMinigames core library.
//!
//! This crate exists so that a front end written in Kotlin or Swift can call the same rules
//! the Rust front end would: it moves values across the boundary and does nothing else. Every
//! decision about what a game does belongs to `qualia_minigames_core`; a rule implemented
//! here would be a second implementation, and the two would drift apart.
//!
//! # The shape of the interface
//!
//! Each game is one exported object holding one game's state, created by a constructor and
//! mutated by its methods. Queries return plain values a front end can draw directly: the
//! side mask of a cell, the indices of the cells on a loop, the cells water reaches.
//!
//! A saved game crosses the boundary as a `Vec<u8>` built by the core's own snapshot format,
//! so the front end stores bytes it never interprets. The format's version is inside those
//! bytes; a snapshot from another version is reported as an error rather than being read.
//!
//! # Errors
//!
//! Reading a snapshot is the only fallible operation, and its failures are reported as
//! [`SnapshotFailure`], a flat enum whose variants say what was wrong. A flat enum rather
//! than the core's own error type because a `#[non_exhaustive]` error cannot cross the
//! boundary: the generated `when` would have no branch for a variant added later.

use std::sync::{Mutex, MutexGuard};

uniffi::setup_scaffolding!();

/// The number of cells on a side of a sudoku board.
const SUDOKU_SIZE: u32 = 9;

/// The number of cells on a sudoku board.
const SUDOKU_CELLS: u32 = 81;

/// Which of the two layouts a pipes board uses.
#[derive(Clone, Copy, Debug, Eq, PartialEq, uniffi::Enum)]
pub enum PipesLayout {
    /// A flat board: a cell on an edge has no neighbour beyond it.
    Flat,
    /// A wrapped board: the opposite edges are joined.
    Wrapped,
}

/// How hard a sudoku puzzle is.
#[derive(Clone, Copy, Debug, Eq, PartialEq, uniffi::Enum)]
pub enum SudokuDifficulty {
    /// Solvable with singles alone.
    Easy,
    /// Needs a locked candidate or a naked pair somewhere.
    Medium,
    /// Needs a hidden pair somewhere.
    Hard,
}

/// Which rules a minesweeper board is generated under.
#[derive(Clone, Copy, Debug, Eq, PartialEq, uniffi::Enum)]
pub enum MinesweeperMode {
    /// A board a player may have to guess on.
    Normal,
    /// A board two deductions can finish, when one is found within the attempt budget.
    NoGuess,
}

/// Where a minesweeper game stands.
#[derive(Clone, Copy, Debug, Eq, PartialEq, uniffi::Enum)]
pub enum MinesweeperPhase {
    /// The board has not been generated yet, because no cell has been opened.
    Pending,
    /// The board is generated and the game is being played.
    Playing,
    /// Every cell that holds no mine is open.
    Won,
    /// A mine was opened.
    Lost,
}

/// What opening a minesweeper cell did.
#[derive(Clone, Copy, Debug, Eq, PartialEq, uniffi::Enum)]
pub enum MinesweeperEvent {
    /// The cell and possibly a flood of its neighbours opened.
    Revealed,
    /// The cell held a mine, which ended the game.
    HitMine,
    /// The last cell that could be opened was opened.
    Won,
    /// The cell could not be opened: the game is over or the cell is open, flagged or outside
    /// the board.
    Ignored,
}

/// What can go wrong while reading a saved game.
///
/// The variants mirror the core's own snapshot errors one for one. They are a separate enum
/// because the core's is `#[non_exhaustive]`, which a generated `when` cannot be written
/// against.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error, uniffi::Error)]
pub enum SnapshotFailure {
    /// The bytes stop before the snapshot's last field.
    #[error("the saved game ends before its last field")]
    Truncated,
    /// The version byte is not one this build reads.
    #[error("the saved game was written by another version of the game ({version})")]
    UnsupportedVersion {
        /// The version the bytes declare.
        version: u32,
    },
    /// A parameter is outside the range the game accepts.
    #[error("the saved game's {field} is out of range: {value}")]
    Parameter {
        /// The name of the parameter.
        field: String,
        /// The value that was read.
        value: u32,
    },
    /// The saved game holds more moves than the format allows.
    #[error("the saved game holds too many moves")]
    TooManyMoves,
    /// A move names a cell outside the board.
    #[error("the saved game moves cell {index}, but the board holds {cells}")]
    MoveOutOfBounds {
        /// The index the move named.
        index: u32,
        /// The number of cells the board has.
        cells: u32,
    },
    /// Bytes are left over after the snapshot's last field.
    #[error("the saved game has trailing bytes")]
    TrailingBytes,
}

impl From<qualia_minigames_core::snapshot::SnapshotError> for SnapshotFailure {
    fn from(error: qualia_minigames_core::snapshot::SnapshotError) -> Self {
        use qualia_minigames_core::snapshot::SnapshotError;
        match error {
            SnapshotError::Truncated => Self::Truncated,
            SnapshotError::UnsupportedVersion(version) => Self::UnsupportedVersion {
                version: u32::from(version),
            },
            SnapshotError::Parameter { field, value } => Self::Parameter {
                field: field.to_owned(),
                value,
            },
            SnapshotError::TooManyMoves => Self::TooManyMoves,
            SnapshotError::MoveOutOfBounds { index, cells } => {
                Self::MoveOutOfBounds { index, cells }
            }
            SnapshotError::TrailingBytes => Self::TrailingBytes,
        }
    }
}

impl From<qualia_minigames_core::pipes::Layout> for PipesLayout {
    fn from(layout: qualia_minigames_core::pipes::Layout) -> Self {
        match layout {
            qualia_minigames_core::pipes::Layout::Flat => Self::Flat,
            qualia_minigames_core::pipes::Layout::Wrapped => Self::Wrapped,
        }
    }
}

impl From<PipesLayout> for qualia_minigames_core::pipes::Layout {
    fn from(layout: PipesLayout) -> Self {
        match layout {
            PipesLayout::Flat => Self::Flat,
            PipesLayout::Wrapped => Self::Wrapped,
        }
    }
}

impl From<SudokuDifficulty> for qualia_minigames_core::sudoku::Difficulty {
    fn from(difficulty: SudokuDifficulty) -> Self {
        match difficulty {
            SudokuDifficulty::Easy => Self::Easy,
            SudokuDifficulty::Medium => Self::Medium,
            SudokuDifficulty::Hard => Self::Hard,
        }
    }
}

impl From<qualia_minigames_core::sudoku::Difficulty> for SudokuDifficulty {
    fn from(difficulty: qualia_minigames_core::sudoku::Difficulty) -> Self {
        match difficulty {
            qualia_minigames_core::sudoku::Difficulty::Easy => Self::Easy,
            qualia_minigames_core::sudoku::Difficulty::Medium => Self::Medium,
            qualia_minigames_core::sudoku::Difficulty::Hard => Self::Hard,
        }
    }
}

impl From<MinesweeperMode> for qualia_minigames_core::minesweeper::Mode {
    fn from(mode: MinesweeperMode) -> Self {
        match mode {
            MinesweeperMode::Normal => Self::Normal,
            MinesweeperMode::NoGuess => Self::NoGuess,
        }
    }
}

impl From<qualia_minigames_core::minesweeper::Mode> for MinesweeperMode {
    fn from(mode: qualia_minigames_core::minesweeper::Mode) -> Self {
        match mode {
            qualia_minigames_core::minesweeper::Mode::Normal => Self::Normal,
            qualia_minigames_core::minesweeper::Mode::NoGuess => Self::NoGuess,
        }
    }
}

impl From<qualia_minigames_core::minesweeper::Phase> for MinesweeperPhase {
    fn from(phase: qualia_minigames_core::minesweeper::Phase) -> Self {
        match phase {
            qualia_minigames_core::minesweeper::Phase::Pending => Self::Pending,
            qualia_minigames_core::minesweeper::Phase::Playing => Self::Playing,
            qualia_minigames_core::minesweeper::Phase::Won => Self::Won,
            qualia_minigames_core::minesweeper::Phase::Lost => Self::Lost,
        }
    }
}

impl From<qualia_minigames_core::minesweeper::Event> for MinesweeperEvent {
    fn from(event: qualia_minigames_core::minesweeper::Event) -> Self {
        match event {
            qualia_minigames_core::minesweeper::Event::Revealed => Self::Revealed,
            qualia_minigames_core::minesweeper::Event::HitMine => Self::HitMine,
            qualia_minigames_core::minesweeper::Event::Won => Self::Won,
            qualia_minigames_core::minesweeper::Event::Ignored => Self::Ignored,
        }
    }
}

/// Returns the version of the core library these bindings were built against.
#[uniffi::export]
pub fn core_version() -> String {
    qualia_minigames_core::VERSION.to_owned()
}

/// Returns the smallest pipes board, in cells on a side.
#[uniffi::export]
pub fn pipes_min_size() -> u32 {
    qualia_minigames_core::pipes::MIN_SIZE
}

/// Returns the largest pipes board, in cells on a side.
#[uniffi::export]
pub fn pipes_max_size() -> u32 {
    qualia_minigames_core::pipes::MAX_SIZE
}

/// Returns the number of cells on a side of a sudoku board.
#[uniffi::export]
pub fn sudoku_size() -> u32 {
    SUDOKU_SIZE
}

/// Turns text into a seed, so a front end can offer a seed a player types.
#[uniffi::export]
pub fn seed_from_text(text: String) -> u64 {
    qualia_minigames_core::random::seed_from_text(&text)
}

/// Returns the smallest minesweeper board, in cells on a side.
#[uniffi::export]
pub fn minesweeper_min_size() -> u32 {
    qualia_minigames_core::minesweeper::MIN_SIZE
}

/// Returns the largest minesweeper board, in cells on a side.
#[uniffi::export]
pub fn minesweeper_max_size() -> u32 {
    qualia_minigames_core::minesweeper::MAX_SIZE
}

/// Returns the lowest share of a board a mine field accepts, in thousandths of a percent.
#[uniffi::export]
pub fn minesweeper_input_per_mille_min() -> u32 {
    qualia_minigames_core::minesweeper::INPUT_PER_MILLE_MIN
}

/// Returns the highest share of a board a mine field accepts, in thousandths of a percent.
#[uniffi::export]
pub fn minesweeper_input_per_mille_max() -> u32 {
    qualia_minigames_core::minesweeper::INPUT_PER_MILLE_MAX
}

/// Returns the lowest share the mine slider covers, in thousandths of a percent.
#[uniffi::export]
pub fn minesweeper_slider_per_mille_min() -> u32 {
    qualia_minigames_core::minesweeper::SLIDER_PER_MILLE_MIN
}

/// Returns the highest share the mine slider covers, in thousandths of a percent.
#[uniffi::export]
pub fn minesweeper_slider_per_mille_max() -> u32 {
    qualia_minigames_core::minesweeper::SLIDER_PER_MILLE_MAX
}

/// Returns the highest share the mine slider covers in no-guess mode, in thousandths of a
/// percent.
#[uniffi::export]
pub fn minesweeper_noguess_slider_per_mille_max() -> u32 {
    qualia_minigames_core::minesweeper::NOGUESS_SLIDER_PER_MILLE_MAX
}

/// Returns the highest share the mine field accepts in no-guess mode, in thousandths of a
/// percent.
#[uniffi::export]
pub fn minesweeper_noguess_input_per_mille_max() -> u32 {
    qualia_minigames_core::minesweeper::NOGUESS_INPUT_PER_MILLE_MAX
}

/// Returns the share from which a no-guess board is likely to fall back, in thousandths of a
/// percent.
#[uniffi::export]
pub fn minesweeper_noguess_warn_per_mille() -> u32 {
    qualia_minigames_core::minesweeper::NOGUESS_WARN_PER_MILLE
}

/// Returns the shares a mine slider snaps to, in thousandths of a percent, ascending.
#[uniffi::export]
pub fn minesweeper_snap_per_mille() -> Vec<u32> {
    qualia_minigames_core::minesweeper::SNAP_PER_MILLE.to_vec()
}

/// Returns how many mines a share of a board converts to.
///
/// The share and the answer are both integers: the share is in thousandths of a percent, so
/// that `12.3%` is `123`, and a border a float rounds the wrong way is where a nine-mine
/// beginner board would come from.
#[uniffi::export]
pub fn minesweeper_mines_for_per_mille(cells: u32, per_mille: u32) -> u32 {
    qualia_minigames_core::minesweeper::mines_for_per_mille(cells, per_mille)
}

/// Returns the seed a date produces, so a puzzle can be the same for everyone that day.
#[uniffi::export]
pub fn daily_seed(date: String) -> u64 {
    qualia_minigames_core::pipes::daily_seed(&date)
}

/// One game of pipes, as the front end holds it.
///
/// The core's game is held behind a mutex because UniFFI shares the object across calls as
/// an `Arc`, which gives no exclusive borrow. The lock covers one call and nothing else: the
/// work inside it is a few hundred bytes of board, never a search.
#[derive(Debug, uniffi::Object)]
pub struct PipesGame {
    /// The core's game, shared through the object.
    game: Mutex<qualia_minigames_core::pipes::Game>,
}

#[uniffi::export]
impl PipesGame {
    /// Generates a puzzle and starts a game on it.
    ///
    /// `size` must be within [`pipes_min_size`] and [`pipes_max_size`]; a value outside that
    /// range is rejected rather than clamped, because a clamped value would start a game on
    /// a board the player did not ask for.
    #[uniffi::constructor]
    pub fn new(size: u32, layout: PipesLayout, seed: u64) -> Result<Self, SnapshotFailure> {
        let minimum = qualia_minigames_core::pipes::MIN_SIZE;
        let maximum = qualia_minigames_core::pipes::MAX_SIZE;
        if !(minimum..=maximum).contains(&size) {
            return Err(SnapshotFailure::Parameter {
                field: "size".to_owned(),
                value: size,
            });
        }
        Ok(Self {
            game: Mutex::new(qualia_minigames_core::pipes::Game::new(
                size,
                layout.into(),
                seed,
            )),
        })
    }

    /// Reads a saved game.
    #[uniffi::constructor]
    pub fn from_snapshot(bytes: Vec<u8>) -> Result<Self, SnapshotFailure> {
        Ok(Self {
            game: Mutex::new(qualia_minigames_core::pipes::Game::from_snapshot(&bytes)?),
        })
    }

    /// Writes the game, so it can be saved and read back.
    pub fn to_snapshot(&self) -> Vec<u8> {
        self.lock().to_snapshot()
    }

    /// Returns the number of cells on a side.
    pub fn size(&self) -> u32 {
        self.lock().board().size()
    }

    /// Returns the layout the board uses.
    pub fn layout(&self) -> PipesLayout {
        self.lock().board().layout().into()
    }

    /// Returns the number of cells on the board.
    pub fn cell_count(&self) -> u32 {
        self.lock().board().cells()
    }

    /// Returns the seed the puzzle was generated from.
    pub fn seed(&self) -> u64 {
        self.lock().seed()
    }

    /// Returns the side mask a cell shows now.
    ///
    /// The mask's bits are the sides the piece joins: [`side_top`], [`side_right`],
    /// [`side_bottom`] and [`side_left`].
    pub fn piece(&self, index: u32) -> u32 {
        u32::from(self.lock().piece(index))
    }

    /// Returns every cell's side mask at once, in row-major order.
    ///
    /// A board is drawn from this rather than from one [`Self::piece`] call per cell: a 35 by 35
    /// board is 1225 crossings of the boundary for one frame, which costs more than the drawing
    /// it feeds. One call returns the same values.
    pub fn pieces(&self) -> Vec<u32> {
        let game = self.lock();
        (0..game.board().cells())
            .map(|index| u32::from(game.piece(index)))
            .collect()
    }

    /// Returns the side mask a cell is solved with.
    pub fn solved_piece(&self, index: u32) -> u32 {
        u32::from(self.lock().board().solved_piece(index))
    }

    /// Turns a cell a quarter turn clockwise.
    pub fn turn_clockwise(&self, index: u32) {
        self.lock().turn_clockwise(index);
    }

    /// Turns a cell a quarter turn anti-clockwise.
    pub fn turn_anti_clockwise(&self, index: u32) {
        self.lock().turn_anti_clockwise(index);
    }

    /// Takes back the last turn.
    ///
    /// Returns whether there was a turn to take back.
    pub fn undo(&self) -> bool {
        self.lock().undo()
    }

    /// Returns how many turns can be taken back.
    pub fn undo_count(&self) -> u32 {
        u32::try_from(self.lock().history_len()).unwrap_or(u32::MAX)
    }

    /// Returns whether the board is one network without a loop.
    pub fn is_solved(&self) -> bool {
        self.lock().is_solved()
    }

    /// Returns every cell that lies on a loop, in ascending order.
    pub fn loop_cells(&self) -> Vec<u32> {
        self.lock().loops()
    }

    /// Returns the cells water reaches from the board's starting cell, in ascending order.
    pub fn water_cells(&self) -> Vec<u32> {
        self.lock().water()
    }

    /// Returns the cell water starts from.
    pub fn water_start(&self) -> u32 {
        qualia_minigames_core::pipes::start_cell(self.lock().board().size())
    }
}

/// The private half of [`PipesGame`].
///
/// It is a separate block on purpose: `#[uniffi::export]` processes every method of the block
/// it is attached to, including private ones, and cannot lower a type that only exists inside
/// Rust — a `MutexGuard` here.
impl PipesGame {
    /// Locks the game for one call.
    fn lock(&self) -> MutexGuard<'_, qualia_minigames_core::pipes::Game> {
        // A poisoned lock means another call panicked while holding it. The board is a plain
        // value with no invariant a panic could break, so the lock is taken back rather than
        // propagated: a front end where every later call fails is worse off than one that
        // sees a partially updated game.
        self.game
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

/// One game of sudoku, as the front end holds it.
#[derive(Debug, uniffi::Object)]
pub struct SudokuGame {
    /// The core's game, shared through the object.
    game: Mutex<qualia_minigames_core::sudoku::Game>,
}

#[uniffi::export]
impl SudokuGame {
    /// Generates a puzzle of a difficulty.
    ///
    /// # Panics
    ///
    /// Panics when the seed produces no puzzle of that difficulty within the attempt budget,
    /// which a generated seed does not do. A seed a player typed goes through
    /// [`sudoku_try_generate`] instead: a panic that crosses a language boundary ends the
    /// process rather than the call.
    #[uniffi::constructor]
    pub fn generate(difficulty: SudokuDifficulty, seed: u64) -> Self {
        Self {
            game: Mutex::new(qualia_minigames_core::sudoku::Game::generate(
                difficulty.into(),
                seed,
            )),
        }
    }

    /// Reads a saved game.
    #[uniffi::constructor]
    pub fn from_snapshot(bytes: Vec<u8>) -> Result<Self, SnapshotFailure> {
        Ok(Self {
            game: Mutex::new(qualia_minigames_core::sudoku::Game::from_snapshot(&bytes)?),
        })
    }

    /// Writes the game, so it can be saved and read back.
    pub fn to_snapshot(&self) -> Vec<u8> {
        self.lock().to_snapshot()
    }

    /// Returns how hard the puzzle is.
    pub fn difficulty(&self) -> SudokuDifficulty {
        self.lock().difficulty().into()
    }

    /// Returns the seed the puzzle was generated from.
    pub fn seed(&self) -> u64 {
        self.lock().seed()
    }

    /// Returns how many cells the board holds.
    pub fn cell_count(&self) -> u32 {
        SUDOKU_CELLS
    }

    /// Returns the digit a cell was given as, or zero for an empty cell.
    pub fn given(&self, index: u32) -> u32 {
        u32::from(self.lock().given(index as usize))
    }

    /// Returns the digit a cell holds now, or zero for an empty cell.
    pub fn digit(&self, index: u32) -> u32 {
        u32::from(self.lock().digit(index as usize))
    }

    /// Returns the candidates noted in a cell, as a mask of the digits `1` to `9`.
    pub fn notes(&self, index: u32) -> u32 {
        u32::from(self.lock().notes(index as usize))
    }

    /// Returns whether a cell was given.
    pub fn is_given(&self, index: u32) -> bool {
        self.lock().is_given(index as usize)
    }

    /// Returns how many digits the puzzle starts with.
    pub fn given_count(&self) -> u32 {
        self.lock().given_count()
    }

    /// Enters a digit into a cell, unless the cell was given.
    ///
    /// An index outside the board or a digit outside `1` to `9` is ignored rather than
    /// rejected: the caller is a screen that only offers those values, and a panic across the
    /// boundary would take down the process instead of one tap.
    pub fn enter(&self, index: u32, digit: u32) {
        if index >= SUDOKU_CELLS || !(1..=9).contains(&digit) {
            return;
        }
        self.lock().enter(index as usize, digit as u8);
    }

    /// Clears a cell the player filled.
    ///
    /// An index outside the board is ignored, for the reason [`SudokuGame::enter`] gives.
    pub fn clear(&self, index: u32) {
        if index >= SUDOKU_CELLS {
            return;
        }
        self.lock().clear(index as usize);
    }

    /// Adds or removes a candidate in an empty cell.
    ///
    /// An index outside the board or a digit outside `1` to `9` is ignored, for the reason
    /// [`SudokuGame::enter`] gives.
    pub fn toggle_note(&self, index: u32, digit: u32) {
        if index >= SUDOKU_CELLS || !(1..=9).contains(&digit) {
            return;
        }
        self.lock().toggle_note(index as usize, digit as u8);
    }

    /// Takes back the last move.
    ///
    /// Returns whether there was a move to take back.
    pub fn undo(&self) -> bool {
        self.lock().undo()
    }

    /// Returns how many moves can be taken back.
    pub fn undo_count(&self) -> u32 {
        u32::try_from(self.lock().move_count()).unwrap_or(u32::MAX)
    }

    /// Returns the cells that break a rule, in ascending order.
    pub fn conflict_cells(&self) -> Vec<u32> {
        self.lock()
            .conflicts()
            .into_iter()
            .map(|cell| cell as u32)
            .collect()
    }

    /// Returns whether the board is filled and every rule holds.
    pub fn is_solved(&self) -> bool {
        self.lock().is_solved()
    }
}

/// The private half of [`SudokuGame`], kept out of the exported block for the reason given on
/// [`PipesGame`]'s.
impl SudokuGame {
    /// Locks the game for one call.
    fn lock(&self) -> MutexGuard<'_, qualia_minigames_core::sudoku::Game> {
        self.game
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

/// Generates a sudoku puzzle, or reports that the seed cannot produce one of the difficulty.
///
/// This is a function rather than a constructor because it can answer nothing: a constructor
/// always builds an object, and a null is what says the seed does not fit. It is what a screen
/// that lets a player type a seed calls, so that a seed which produces no puzzle of the chosen
/// difficulty leaves the form open with a message rather than taking the process down with a
/// panic from the infallible [`SudokuGame::generate`].
#[uniffi::export]
pub fn sudoku_try_generate(
    difficulty: SudokuDifficulty,
    seed: u64,
) -> Option<std::sync::Arc<SudokuGame>> {
    qualia_minigames_core::sudoku::Game::try_generate(difficulty.into(), seed).map(|game| {
        std::sync::Arc::new(SudokuGame {
            game: Mutex::new(game),
        })
    })
}

/// One game of minesweeper, as the front end holds it.
///
/// The board is generated lazily for a random game: the first cell a player opens is what
/// lays the mines down, which is what makes that cell safe. A game started from an explicit
/// seed is generated when it is created instead.
#[derive(Debug, uniffi::Object)]
pub struct MinesweeperGame {
    /// The core's game, shared through the object.
    game: Mutex<qualia_minigames_core::minesweeper::Game>,
}

#[uniffi::export]
impl MinesweeperGame {
    /// Starts a game whose board is generated when the first cell is opened.
    ///
    /// The mine count is bounded by the cells outside the first click's protected window: a
    /// game that promised a safe first click and laid a mine in it would be a game that broke
    /// its own rule.
    #[uniffi::constructor]
    pub fn random(
        width: u32,
        height: u32,
        mines: u32,
        mode: MinesweeperMode,
        base_seed: u64,
    ) -> Result<Self, SnapshotFailure> {
        Ok(Self {
            game: Mutex::new(qualia_minigames_core::minesweeper::Game::random(
                width,
                height,
                mines,
                mode.into(),
                base_seed,
            )?),
        })
    }

    /// Starts a game on a board generated from an explicit seed.
    ///
    /// The board exists at once and its first click is not protected, which is what a seed a
    /// player typed means: the same seed always opens the same board.
    #[uniffi::constructor]
    pub fn from_seed(
        width: u32,
        height: u32,
        mines: u32,
        mode: MinesweeperMode,
        seed: u64,
    ) -> Result<Self, SnapshotFailure> {
        Ok(Self {
            game: Mutex::new(qualia_minigames_core::minesweeper::Game::from_seed(
                width,
                height,
                mines,
                mode.into(),
                seed,
            )?),
        })
    }

    /// Reads a saved game.
    #[uniffi::constructor]
    pub fn from_snapshot(bytes: Vec<u8>) -> Result<Self, SnapshotFailure> {
        Ok(Self {
            game: Mutex::new(qualia_minigames_core::minesweeper::Game::from_snapshot(
                &bytes,
            )?),
        })
    }

    /// Writes the game, so it can be saved and read back.
    pub fn to_snapshot(&self) -> Vec<u8> {
        self.lock().to_snapshot()
    }

    /// Returns the number of cells across.
    pub fn width(&self) -> u32 {
        self.lock().width()
    }

    /// Returns the number of cells down.
    pub fn height(&self) -> u32 {
        self.lock().height()
    }

    /// Returns the number of cells on the board.
    pub fn cell_count(&self) -> u32 {
        self.lock().cells()
    }

    /// Returns how many mines the board holds.
    pub fn mine_count(&self) -> u32 {
        self.lock().mine_count()
    }

    /// Returns how many cells are flagged.
    pub fn flag_count(&self) -> u32 {
        self.lock().flag_count()
    }

    /// Returns how many mines the flags leave unaccounted for.
    ///
    /// The answer goes negative when more cells are flagged than the board holds mines, which
    /// the game allows and a counter is expected to show.
    pub fn remaining_mines(&self) -> i32 {
        self.lock().remaining_mines()
    }

    /// Returns where the game stands.
    pub fn phase(&self) -> MinesweeperPhase {
        self.lock().phase().into()
    }

    /// Returns the rules the board was asked for.
    pub fn mode(&self) -> MinesweeperMode {
        self.lock().mode().into()
    }

    /// Returns whether a no-guess board could not be found and the game fell back to normal.
    pub fn fell_back(&self) -> bool {
        self.lock().fell_back()
    }

    /// Returns the seed the board was generated from, or zero while none has been.
    pub fn seed(&self) -> u64 {
        self.lock().seed()
    }

    /// Returns what a cell shows now.
    ///
    /// The value is the number of neighbouring mines for an open cell that holds none, or
    /// [`MinesweeperGame::cells_state`]'s code for a covered, flagged or exploded cell.
    /// An index outside the board answers `9`, which is what a covered cell answers: a cell
    /// that does not exist and a cell nobody has opened look the same to a drawing.
    pub fn cell(&self, index: u32) -> u8 {
        let game = self.lock();
        if index >= game.cells() {
            return qualia_minigames_core::minesweeper::COVERED;
        }
        game.cell(index)
    }

    /// Returns every cell's state at once, in row-major order.
    ///
    /// A board is drawn from this rather than from one [`Self::cell`] call per cell: a 128 by
    /// 128 board is 16384 crossings of the language boundary for one frame, which costs more
    /// than the drawing it feeds. The values are the neighbouring mine count for an open cell
    /// that holds none, `9` for a covered cell, `10` for a flagged one and `11` for an open
    /// mine.
    pub fn cells_state(&self) -> Vec<u8> {
        self.lock().cells_state()
    }

    /// Opens a cell.
    ///
    /// An index outside the board is ignored rather than rejected: the caller is a screen
    /// that only offers cells of the board, and a panic across the boundary would take down
    /// the process instead of one tap.
    pub fn reveal(&self, index: u32) -> MinesweeperEvent {
        self.lock().reveal(index).into()
    }

    /// Flags a covered cell, or takes the flag off a flagged one.
    ///
    /// Returns whether the cell was changed. An index outside the board changes nothing.
    pub fn toggle_flag(&self, index: u32) -> bool {
        self.lock().toggle_flag(index)
    }
}

/// The private half of [`MinesweeperGame`], kept out of the exported block for the reason
/// given on [`PipesGame`]'s.
impl MinesweeperGame {
    /// Locks the game for one call.
    fn lock(&self) -> MutexGuard<'_, qualia_minigames_core::minesweeper::Game> {
        self.game
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

/// Returns the bit that stands for a cell's top side.
#[uniffi::export]
pub fn side_top() -> u32 {
    u32::from(qualia_minigames_core::pipes::side::TOP)
}

/// Returns the bit that stands for a cell's right side.
#[uniffi::export]
pub fn side_right() -> u32 {
    u32::from(qualia_minigames_core::pipes::side::RIGHT)
}

/// Returns the bit that stands for a cell's bottom side.
#[uniffi::export]
pub fn side_bottom() -> u32 {
    u32::from(qualia_minigames_core::pipes::side::BOTTOM)
}

/// Returns the bit that stands for a cell's left side.
#[uniffi::export]
pub fn side_left() -> u32 {
    u32::from(qualia_minigames_core::pipes::side::LEFT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_pipes_game_crosses_the_boundary_with_its_state() {
        let game = PipesGame::new(9, PipesLayout::Wrapped, 7).expect("9 is a valid size");
        assert_eq!(game.size(), 9);
        assert_eq!(game.cell_count(), 81);
        assert_eq!(game.layout(), PipesLayout::Wrapped);
        let before = game.piece(12);
        game.turn_clockwise(12);
        assert_ne!(game.piece(12), before);
        assert_eq!(game.undo_count(), 1);
        assert!(game.undo());
        assert_eq!(game.piece(12), before);
    }

    #[test]
    fn every_piece_comes_back_in_one_call() {
        // The batched read has to agree with the per-cell one: it exists only to save
        // crossings, and a batch that disagreed with the cell it replaced would draw a board
        // that the rules did not describe.
        let game = PipesGame::new(7, PipesLayout::Wrapped, 11).expect("7 is a valid size");
        game.turn_clockwise(3);
        game.turn_anti_clockwise(20);
        let batch = game.pieces();
        assert_eq!(batch.len(), 49);
        for (index, mask) in batch.iter().enumerate() {
            assert_eq!(*mask, game.piece(index as u32), "cell {index}");
        }
    }

    #[test]
    fn a_pipes_game_refuses_a_size_outside_the_range() {
        let error = PipesGame::new(3, PipesLayout::Flat, 0).unwrap_err();
        assert_eq!(
            error,
            SnapshotFailure::Parameter {
                field: "size".to_owned(),
                value: 3
            }
        );
        let error = PipesGame::new(36, PipesLayout::Flat, 0).unwrap_err();
        assert_eq!(
            error,
            SnapshotFailure::Parameter {
                field: "size".to_owned(),
                value: 36
            }
        );
    }

    #[test]
    fn a_pipes_snapshot_survives_the_boundary() {
        let game = PipesGame::new(12, PipesLayout::Flat, 3).expect("12 is a valid size");
        game.turn_clockwise(4);
        game.turn_anti_clockwise(4);
        game.turn_anti_clockwise(4);
        let restored = PipesGame::from_snapshot(game.to_snapshot()).expect("it is read");
        assert_eq!(restored.piece(4), game.piece(4));
        assert_eq!(restored.undo_count(), game.undo_count());
    }

    #[test]
    fn a_broken_pipes_snapshot_is_reported_as_an_error() {
        let error = PipesGame::from_snapshot(vec![0, 1, 2]).unwrap_err();
        assert!(matches!(error, SnapshotFailure::UnsupportedVersion { .. }));
    }

    #[test]
    fn the_side_masks_agree_with_the_core() {
        assert_eq!(side_top(), 1);
        assert_eq!(side_right(), 2);
        assert_eq!(side_bottom(), 4);
        assert_eq!(side_left(), 8);
    }

    #[test]
    fn a_sudoku_game_crosses_the_boundary_with_its_state() {
        let game = SudokuGame::generate(SudokuDifficulty::Easy, 5);
        assert_eq!(game.difficulty(), SudokuDifficulty::Easy);
        assert_eq!(game.cell_count(), 81);
        let empty = (0..SUDOKU_CELLS)
            .find(|cell| !game.is_given(*cell))
            .expect("an empty cell");
        game.enter(empty, 4);
        assert_eq!(game.digit(empty), 4);
        assert!(game.undo());
        assert_eq!(game.digit(empty), 0);
    }

    #[test]
    fn a_sudoku_snapshot_survives_the_boundary() {
        let game = SudokuGame::generate(SudokuDifficulty::Hard, 9);
        let empty = (0..SUDOKU_CELLS)
            .find(|cell| !game.is_given(*cell))
            .expect("an empty cell");
        game.toggle_note(empty, 6);
        let restored = SudokuGame::from_snapshot(game.to_snapshot()).expect("it is read");
        assert_eq!(restored.difficulty(), SudokuDifficulty::Hard);
        assert_eq!(restored.notes(empty), game.notes(empty));
        assert_eq!(restored.undo_count(), game.undo_count());
    }

    #[test]
    fn a_sudoku_given_cannot_be_changed_across_the_boundary() {
        let game = SudokuGame::generate(SudokuDifficulty::Medium, 11);
        let given = (0..SUDOKU_CELLS)
            .find(|cell| game.is_given(*cell))
            .expect("a given");
        let before = game.digit(given);
        game.enter(given, if before == 9 { 1 } else { 9 });
        assert_eq!(game.digit(given), before);
        assert_eq!(game.undo_count(), 0);
    }

    #[test]
    fn a_sudoku_entry_outside_the_digits_is_ignored() {
        let game = SudokuGame::generate(SudokuDifficulty::Easy, 13);
        let empty = (0..SUDOKU_CELLS)
            .find(|cell| !game.is_given(*cell))
            .expect("an empty cell");
        game.enter(empty, 0);
        assert_eq!(game.digit(empty), 0);
        game.enter(empty, 10);
        assert_eq!(game.digit(empty), 0);
    }

    #[test]
    fn a_broken_sudoku_snapshot_is_reported_as_an_error() {
        let error = SudokuGame::from_snapshot(vec![0]).unwrap_err();
        assert_eq!(error, SnapshotFailure::UnsupportedVersion { version: 0 });
    }

    #[test]
    fn a_seed_can_be_typed_as_text() {
        assert_eq!(
            seed_from_text("example".to_owned()),
            seed_from_text("example".to_owned())
        );
        assert_ne!(
            seed_from_text("example".to_owned()),
            seed_from_text("another".to_owned())
        );
        assert_eq!(
            daily_seed("2026-10-29".to_owned()),
            daily_seed("2026-10-29".to_owned())
        );
    }

    #[test]
    fn a_sudoku_generation_can_report_that_a_seed_does_not_fit() {
        // The fallible entry point answers the same puzzle as the infallible one when the seed
        // does fit, which is what a screen relies on when it offers a seed at all.
        let generated =
            sudoku_try_generate(SudokuDifficulty::Easy, 5).expect("5 fits an easy puzzle");
        let expected = SudokuGame::generate(SudokuDifficulty::Easy, 5);
        assert_eq!(generated.seed(), expected.seed());
        assert_eq!(generated.given_count(), expected.given_count());
        assert_eq!(generated.to_snapshot(), expected.to_snapshot());
    }

    #[test]
    fn a_minesweeper_game_waits_for_its_first_click() {
        let game = MinesweeperGame::random(9, 9, 10, MinesweeperMode::Normal, 7)
            .expect("9 by 9 at 10 mines is valid");
        assert_eq!(game.width(), 9);
        assert_eq!(game.height(), 9);
        assert_eq!(game.cell_count(), 81);
        assert_eq!(game.mine_count(), 10);
        assert_eq!(game.phase(), MinesweeperPhase::Pending);
        assert_eq!(game.seed(), 0);
        assert_eq!(game.remaining_mines(), 10);
        assert!(
            game.cells_state().iter().all(|state| *state == 9),
            "a pending board is covered"
        );
    }

    #[test]
    fn the_first_click_of_a_random_game_lays_the_mines_down_safely() {
        let game = MinesweeperGame::random(9, 9, 10, MinesweeperMode::Normal, 7)
            .expect("the parameters are valid");
        assert_eq!(game.reveal(40), MinesweeperEvent::Revealed);
        assert_eq!(game.phase(), MinesweeperPhase::Playing);
        assert_ne!(game.seed(), 0);
        let opened = game.cell(40);
        assert!(opened <= 8, "the first click opened a mine: {opened}");
    }

    #[test]
    fn a_game_started_from_a_seed_exists_at_once() {
        let game = MinesweeperGame::from_seed(9, 9, 10, MinesweeperMode::Normal, 0x5eed)
            .expect("the parameters are valid");
        assert_eq!(game.phase(), MinesweeperPhase::Playing);
        assert_eq!(game.seed(), 0x5eed);
        let batch = game.cells_state();
        assert_eq!(batch.len(), 81);
        for (index, state) in batch.iter().enumerate() {
            assert_eq!(*state, game.cell(index as u32), "cell {index}");
        }
    }

    #[test]
    fn a_mine_ends_the_game_across_the_boundary() {
        // The mine is found by opening cells of a fresh board one at a time: a board started
        // from a seed lays its mines down at once, so the cell that explodes is a fact of the
        // seed and the test reads it rather than knowing it.
        let mine = (0..81)
            .find(|index| {
                let probe = MinesweeperGame::from_seed(9, 9, 10, MinesweeperMode::Normal, 5)
                    .expect("the parameters are valid");
                probe.reveal(*index) == MinesweeperEvent::HitMine
            })
            .expect("a 10-mine board holds a cell that explodes");
        let game = MinesweeperGame::from_seed(9, 9, 10, MinesweeperMode::Normal, 5)
            .expect("the parameters are valid");
        assert_eq!(game.reveal(mine), MinesweeperEvent::HitMine);
        assert_eq!(game.phase(), MinesweeperPhase::Lost);
        assert_eq!(game.reveal(0), MinesweeperEvent::Ignored);
    }

    #[test]
    fn a_flag_changes_the_remaining_count_across_the_boundary() {
        let game = MinesweeperGame::random(9, 9, 10, MinesweeperMode::Normal, 3)
            .expect("the parameters are valid");
        assert_eq!(game.remaining_mines(), 10);
        assert!(game.toggle_flag(0));
        assert_eq!(game.flag_count(), 1);
        assert_eq!(game.remaining_mines(), 9);
    }

    #[test]
    fn a_minesweeper_snapshot_survives_the_boundary() {
        let game = MinesweeperGame::random(16, 16, 40, MinesweeperMode::NoGuess, 21)
            .expect("the parameters are valid");
        game.reveal(120);
        game.toggle_flag(0);
        let restored = MinesweeperGame::from_snapshot(game.to_snapshot()).expect("it is read");
        assert_eq!(restored.cells_state(), game.cells_state());
        assert_eq!(restored.mode(), game.mode());
        assert_eq!(restored.fell_back(), game.fell_back());
        assert_eq!(restored.remaining_mines(), game.remaining_mines());
    }

    #[test]
    fn a_minesweeper_game_refuses_a_size_or_a_mine_count_outside_its_range() {
        let error = MinesweeperGame::random(3, 9, 5, MinesweeperMode::Normal, 0).unwrap_err();
        assert!(matches!(error, SnapshotFailure::Parameter { .. }));
        let error = MinesweeperGame::random(129, 9, 5, MinesweeperMode::Normal, 0).unwrap_err();
        assert!(matches!(error, SnapshotFailure::Parameter { .. }));
        let error = MinesweeperGame::random(9, 9, 0, MinesweeperMode::Normal, 0).unwrap_err();
        assert!(matches!(error, SnapshotFailure::Parameter { .. }));
        // A 4 by 4 board protects four cells, so twelve mines fit at random and thirteen do
        // not, which is the bound a promised safe first click puts on a game.
        assert!(MinesweeperGame::random(4, 4, 12, MinesweeperMode::Normal, 0).is_ok());
        assert!(MinesweeperGame::random(4, 4, 13, MinesweeperMode::Normal, 0).is_err());
    }

    #[test]
    fn a_minesweeper_index_outside_the_board_is_ignored() {
        let game = MinesweeperGame::random(9, 9, 10, MinesweeperMode::Normal, 3)
            .expect("the parameters are valid");
        assert_eq!(game.reveal(81), MinesweeperEvent::Ignored);
        assert!(!game.toggle_flag(81));
        assert_eq!(
            game.cell(81),
            9,
            "a cell that does not exist reads as covered"
        );
    }

    #[test]
    fn a_broken_minesweeper_snapshot_is_reported_as_an_error() {
        let error = MinesweeperGame::from_snapshot(vec![0]).unwrap_err();
        assert_eq!(error, SnapshotFailure::UnsupportedVersion { version: 0 });
    }

    #[test]
    fn the_minesweeper_constants_agree_with_the_core() {
        assert_eq!(minesweeper_min_size(), 4);
        assert_eq!(minesweeper_max_size(), 128);
        assert_eq!(minesweeper_input_per_mille_min(), 100);
        assert_eq!(minesweeper_input_per_mille_max(), 250);
        assert_eq!(minesweeper_slider_per_mille_min(), 120);
        assert_eq!(minesweeper_slider_per_mille_max(), 250);
        assert_eq!(minesweeper_noguess_slider_per_mille_max(), 156);
        assert_eq!(minesweeper_noguess_input_per_mille_max(), 206);
        assert_eq!(minesweeper_noguess_warn_per_mille(), 180);
        assert_eq!(
            minesweeper_snap_per_mille(),
            vec![120, 123, 150, 156, 180, 206, 220, 250]
        );
        assert_eq!(minesweeper_mines_for_per_mille(81, 123), 10);
        assert_eq!(minesweeper_mines_for_per_mille(256, 156), 40);
        assert_eq!(minesweeper_mines_for_per_mille(480, 206), 99);
    }

    #[test]
    fn the_reported_version_is_the_core_version() {
        assert_eq!(core_version(), qualia_minigames_core::VERSION);
    }
}
