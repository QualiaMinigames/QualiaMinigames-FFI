<div align="center" style="align-items: center">

# QualiaMinigames-FFI

[简体中文](README.md) | English

The UniFFI bindings of QualiaMinigames.

Exposes the rules of `qualia-minigames-core` to Kotlin and Swift front ends: values cross the
boundary, decisions do not.

[![License](https://img.shields.io/github/license/QualiaMinigames/QualiaMinigames-FFI?label=License)][License]
[![Releases](https://img.shields.io/github/downloads/QualiaMinigames/QualiaMinigames-FFI/total?logo=github&label=Downloads)][Releases]

[License]: LICENSE

[Releases]: https://github.com/QualiaMinigames/QualiaMinigames-FFI/releases

</div>

---

## Status

| Item | Value |
|---|---|
| crate | `qualia-minigames-ffi` |
| Version | `0.3.0` |
| Type | `cdylib` + `staticlib` |
| Edition | 2024 |
| MSRV | **1.89** |
| Toolchain | `stable`, pinned by `rust-toolchain.toml` |
| License | `LGPL-3.0-or-later` |
| UniFFI | `0.32.2` |

## Usage

A front end generates the bindings at build time and then loads the dynamic library:

```powershell
uniffi-bindgen generate --library target\release\qualia_minigames_ffi.dll --language kotlin --out-dir build\bindings
```

The generated Kotlin lands in `build/bindings/com/qiushui1012/app/minigames/ffi/`.
`PipesGame`, `SudokuGame`, `MinesweeperGame`, `NonogramGame` and `HashiGame` are the five
games, and `uniffiEnsureInitialized()` must run before the first call.

## Interface

Pipes exports the board size, the layout, the side mask of every cell, turning, undo, the
solved check, the cells on a loop, the cells water reaches and the water's starting cell.
Sudoku exports the givens, the current digits, the candidate masks, entering, clearing,
toggling a note, undo, the cells in conflict and the solved check. Minesweeper exports the
width and height, the mine and flag counts, the mines left, the phase, the mode, the fallback
flag, the seed, one cell's state and the whole board's state, opening a cell and flagging one;
mine densities cross the boundary as integers in thousandths of a percent, and the core does
the conversion.

Nonogram exports the width and height, the seed, the fallback flag, both clue lists as
`ClueLine` records, the whole board's cell states, the rows and columns whose clues can no
longer match, filling and marking a cell, and the solved check. Hashi exports the width and
height, the island count, the seed, every island as an `IslandInfo` record, every joined pair
as a `BridgeInfo` record, connecting two islands, toggling a player's mark and the solved
check. Both expose their size ranges through exported constants, and hashi also exposes the
slider's snap table and the conversion from a share of the board to an island count.

Every game writes itself to bytes and reads itself back; the format belongs to the core, and
the front end stores it without interpreting it.

Reading a snapshot is the only operation that fails, and it fails through `SnapshotFailure`.
Caller mistakes such as a digit outside `1` to `9` are ignored rather than thrown: a panic
across the language boundary takes down the process, while a stray tap should only be
dropped. A sudoku seed that produces no puzzle of the chosen difficulty makes
`sudokuTryGenerate` answer null rather than panic, for the same reason: choosing a seed is not
a reason to end the process.

## Design

This crate moves values and nothing else. Any parsing, validation or rule that appears here
is a design regression: every rule lives in `qualia-minigames-core`, and every value a front
end sees comes from there.

## Contributing

Everything must be green before a commit:

```powershell
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo doc --no-deps            # with RUSTDOCFLAGS="-D warnings"
cargo deny check
```

The dependency rules match `qualia-minigames-core`. Commit messages are in English and follow
[Conventional Commits](https://www.conventionalcommits.org/).

## License

Licensed under the **GNU Lesser General Public License v3.0 or later**
(`LGPL-3.0-or-later`), without any warranty, including the implied warranty of
merchantability or fitness for a particular purpose. See `COPYING.LESSER` and `LICENSE`.

The licenses of the dependencies are listed in `NOTICE`.
