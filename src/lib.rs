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
    fn the_reported_version_is_the_core_version() {
        assert_eq!(core_version(), qualia_minigames_core::VERSION);
    }
}
