<div align="center" style="align-items: center">

# QualiaMinigames-FFI

简体中文 | [English](README.en.md)

QualiaMinigames 的 UniFFI 绑定层。

把 `qualia-minigames-core` 的规则暴露给 Kotlin 与 Swift 前端：跨界只搬值，不做判断。

[![License](https://img.shields.io/github/license/QualiaMinigames/QualiaMinigames-FFI?label=License)][License]
[![Releases](https://img.shields.io/github/downloads/QualiaMinigames/QualiaMinigames-FFI/total?logo=github&label=Downloads)][Releases]

[License]: LICENSE

[Releases]: https://github.com/QualiaMinigames/QualiaMinigames-FFI/releases

</div>

---

## 状态

| 项 | 值 |
|---|---|
| crate 名 | `qualia-minigames-ffi` |
| 版本 | `0.2.0` |
| 类型 | `cdylib` + `staticlib` |
| Edition | 2024 |
| MSRV | **1.89** |
| 开发工具链 | `stable`（由 `rust-toolchain.toml` 钉住） |
| 许可证 | `LGPL-3.0-or-later` |
| UniFFI | `0.32.2` |

## 用法

前端构建时先生成绑定，再加载动态库：

```powershell
uniffi-bindgen generate --library target\release\qualia_minigames_ffi.dll --language kotlin --out-dir build\bindings
```

生成的 Kotlin 文件位于 `build/bindings/com/qiushui1012/app/minigames/ffi/`，其中的
`PipesGame`、`SudokuGame` 与 `MinesweeperGame` 是三个游戏的对象，`uniffiEnsureInitialized()`
必须在前端第一次调用之前执行。

## 接口范围

接水管导出棋盘尺寸、布局、每格的边掩码、旋转、撤销、完成判定、环路格集合、水流格集合
与水流起点。数独导出给定值、当前值、候选掩码、进入、清除、笔记开关、撤销、冲突格集合
与完成判定。扫雷导出宽高、雷数、旗数、剩余雷数、阶段、模式、回退标记、种子、单格状态
与批量状态、翻开、标记；雷数密度以千分之一点的整数跨边界传递，换算由 Core 完成。三者
都能把自己写成字节数组，并由字节数组读回；字节格式由 Core 定义，前端只保存不解释。

读取存档是唯一会失败的操作，失败以 `SnapshotFailure` 返回。数字越界这类调用方错误不会
抛出异常，而是被忽略：跨语言边界的 panic 会终止整个进程，而一个越界的点击只应被丢弃。
数独的种子无法产出所选难度时，`sudokuTryGenerate` 返回 null 而不是 panic，同样是为了
不让一次开局选择终止进程。

## 设计约束

本 crate 只做搬运：任何加密、解析或校验逻辑出现在这里都属于设计倒退。规则全部位于
`qualia-minigames-core`，前端看到的每个值都来自那里。

## 参与贡献

提交前请确保全绿：

```powershell
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo doc --no-deps            # 需带 RUSTDOCFLAGS="-D warnings"
cargo deny check
```

依赖的规则与 `qualia-minigames-core` 一致。提交信息使用英文并遵守
[Conventional Commits](https://www.conventionalcommits.org/)。

## 许可

本项目按 **GNU Lesser General Public License v3.0 or later**
（`LGPL-3.0-or-later`）授权。本作品出于希望其有用的目的分发，但不提供任何担保，
甚至不提供适销性或特定用途适用性的默示担保。详见 `COPYING.LESSER` 与 `LICENSE`。

依赖的许可证见 `NOTICE`。
