# RMojo 開発方針 / Development notes

**日本語**: 共通ルールは [`open-raid-z/CLAUDE.md`](https://github.com/aon-co-jp/open-raid-z) と [`runo`](https://github.com/aon-co-jp/runo) の `CLAUDE.md` を参照。チャットは日本語のみ。確認なしで開発・TEST・DEBUG・commit・push。UI・インストーラーはリリース前に実機/エミュレーターで確認。heredocでバックスラッシュを含むコードを書かない(Writeツールを使う)。
**English**: shared rules live in `open-raid-z/CLAUDE.md` and `runo/CLAUDE.md`. Chat in Japanese only. Develop, test, debug, commit and push without asking. Verify UI/installers on real devices/emulators before each release. Do not write code containing backslashes through bash heredocs (use the Write tool).

## 状態 / Status
- 日本語: v0.3.0。Windows x64インストーラー・Android実機(Android 14)・Android TVエミュレーター(i686)で確認済み。Windows ARM64(Snapdragon)ネイティブビルドとRMojo-Setup-arm64.exeをCIに追加(CI成功、実機未確認)。
- English: v0.3.0. Verified on the Windows x64 installer, a real Android 14 phone and the Android TV emulator (i686). Native Windows ARM64 (Snapdragon) build and RMojo-Setup-arm64.exe added in CI (CI green, no real device).

## 未了 / Open
- 日本語: Snapdragon実機、BSD実機、iOS実行パッケージ、リスト多用時の速度(CPythonの約3.5倍遅い)、dict内包・while/else・f-string等、WSLでのAndroidビルド(Linux版NDKが必要。Windows側cargoでは scripts/android-i686-wsl.sh の代わりに cargo ndk -t x86)。
- English: Snapdragon hardware, BSD hardware, iOS runnable package, list-heavy speed (~3.5x slower than CPython), dict comprehensions / while-else / f-strings, Android build in WSL (needs a Linux NDK; with Windows cargo use cargo ndk -t x86).

詳細・再開用メッセージ / details and the resume message: [`PORTING.md`](PORTING.md)
