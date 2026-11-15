# RMojo PORTING / 再開メモ

## 日本語
**状態**: v0.3.0。Windows x64インストーラー・Android実機(Android 14)・Android TVエミュレーター(i686)で確認済み。Windows ARM64(Snapdragon)ネイティブビルドとRMojo-Setup-arm64.exeをCIに追加(CI成功、実機未確認)。
**未了**: Snapdragon実機、BSD実機、iOS実行パッケージ、リスト多用時の速度(CPythonの約3.5倍遅い)、dict内包・while/else・f-string等、WSLでのAndroidビルド(Linux版NDKが必要。Windows側cargoでは scripts/android-i686-wsl.sh の代わりに cargo ndk -t x86)。
**再開用メッセージ**: 「`F:\runo\PORTING.md` 末尾の「2026-11-15 別アカウントでの再開用メモ」と、`F:$1\PORTING.md` を読んで、未了の続きを確認なしで開発・TEST・DEBUG・commit・pushして。チャットは日本語のみ。」
**テスト**: python tests/run.py target/release/rmojo (CPythonとの差分 / differential vs CPython)

## English
**Status**: v0.3.0. Verified on the Windows x64 installer, a real Android 14 phone and the Android TV emulator (i686). Native Windows ARM64 (Snapdragon) build and RMojo-Setup-arm64.exe added in CI (CI green, no real device).
**Open**: Snapdragon hardware, BSD hardware, iOS runnable package, list-heavy speed (~3.5x slower than CPython), dict comprehensions / while-else / f-strings, Android build in WSL (needs a Linux NDK; with Windows cargo use cargo ndk -t x86).
**Resume message**: "Read the 2026-11-15 section at the end of `F:\runo\PORTING.md` and `F:$1\PORTING.md`, then continue the open items without asking (develop, test, debug, commit, push). Chat in Japanese only."
**Tests**: python tests/run.py target/release/rmojo (CPythonとの差分 / differential vs CPython)
