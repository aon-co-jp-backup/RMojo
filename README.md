# RMojo

**日本語**: RMojo — Mojoとの互換性を重視した言語。見た目はPython構文、中身はRust(C++なし)。旧 MojoW / MojoL / MojoM / MojoA / Mojoi(旧称 MojoU)を1つに統合したもの。対象: Windows・Linux・UNIX・macOS・TV(アプリのダウンロード機能がある場合)・Android・iPhone・iPad(**目標**。検証済みの範囲は下の表と制限を参照)。
プラットフォームごとの違いは**インストーラーだけ**(ソースは共通)。

**English**: RMojo - a language that aims for Mojo compatibility: Python syntax on the surface, memory-safe Rust inside (no C++). Successor of MojoW / MojoL / MojoM / MojoA / Mojoi (formerly named MojoU). Targets: Windows, Linux, UNIX, macOS, TV (where apps can be downloaded), Android, iPhone, iPad (goals; see the table and limits for what is verified). Platforms differ only in the installer.

```
rmojo run hello.py        rmojo --version
```

## Installers
| Platform | Installer | Status |
|---|---|---|
| Windows x64 | `installers/windows/rmojo.iss` -> `RMojo-Setup.exe` (per-user, PATH) | **インストール/実行/アンインストール試験OK** (PATH追加・削除も確認) |
| Linux x86_64 / aarch64 | `installers/linux/install.sh` (tar.gz) | built by CI |
| macOS Intel / Apple silicon | `installers/macos/install.sh` (tar.gz) | built by CI, unsigned |
| Android aarch64 | `installers/android/install.sh` (Termux / adb) | **実機で確認** (moto g53y, Android 14: v0.2.0 の version / .mojo / .py 実行OK) |
| iOS | `installers/ios/README.md` | compile check only |

`compat/wsl-bridge/` keeps the old MojoW feature (run the *official* Mojo inside WSL from Windows; unofficial).

## Scope / limits (honest)
Subset of Python syntax; 64-bit integers (overflow is an error); no lists/dicts/classes/closures yet. Typed `int -> int` functions are compiled
(about 1.6-1.9x faster than CPython 3.14 on fib/collatz); untyped code is slower than CPython. Tests: `tests/run.py` compares output with CPython (differences=0).

**Mojo互換 / Mojo compatibility (honest)**: 現状は Python構文のサブセットで、Mojoの `fn` / `struct` / `var` / SIMD 等は未対応。互換はこれから広げる目標。 / Currently a Python-syntax subset; Mojo's `fn` / `struct` / `var` / SIMD are not supported yet. Compatibility is a goal to be widened. 未検証: UNIX(BSD等)、TV、iOS/Androidの実機。 / Unverified: BSD/UNIX, TV, real iOS/Android devices.

**Mojo構文 / Mojo syntax**: `.mojo` ファイルで次が動く(`src/mojo_syntax.rs`、テストは `tests/mojo_*.mojo` と `*.expected`)。 / Working in `.mojo` files:
- `fn` / `var` / `let` / `alias` / 引数規約(`inout` 等) / 型名(Int, Float64, String, Bool)
- `struct`(`var`フィールド、`fn`メソッド、`__init__`、`__str__`=`print`に反映、`@value`=フィールド順のコンストラクタ)
- **値渡し(コピー)**: `var b = a` はリスト・辞書・structを深いコピーにする(`.py`ファイルは従来どおり参照)
- `trait` ブロックと適合リスト `struct P(Shape)` は受け付けるが**適合の検査はしない**(ダックタイピング)
- `List[T](...)` / `Dict[K, V]()` / `SIMD[T, n](...)`(要素ごとの演算、`reduce_add`、添字。**SIMD命令ではなく通常の計算**であり高速化はしない)
- リスト・辞書・文字列メソッド(append/pop/insert/sort/get/keys/values/items/split/join等)、`for x in リスト/辞書/文字列`、`in`、`len/sum/sorted/list`(`tests/t3_collections.py`はCPythonと差分0)
- **未対応**: スライス、タプル型、パラメータ化(`fn f[T: ...]`・`struct S[T]`)、他のデコレーター、ポインタ、Mojoモジュールのimport、`def`の引数型検査(型は注釈のみで検査しない)、整数は64bit。 / Not supported: slices, tuples as a type, parametric fn/struct, other decorators, pointers, Mojo module imports, type checking of annotations; integers are 64-bit.

**BSD/UNIX**: FreeBSD・NetBSD・illumos向けに `cargo check` が通る(`scripts/bsd-check.sh`)。実機・VMでの実行は未確認。 / `cargo check` passes for FreeBSD, NetBSD and illumos; not run on real systems. **Android TV**: エミュレーター(Android 13 TV x86、leanback)で v0.2.0 の `.mojo`/`.py` 実行を確認(i686-linux-android ビルド)。TV実機は未確認。 / **iPhone/iPad**: コンパイル確認のみ。
