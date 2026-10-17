# Scoot

Windows 向けのシンプルなコマンドランチャーアプリ。

- ホットキー(デフォルト `Alt+Space`)で呼び出し
- キーワードでfuzzy search、Tab/Shift-Tabで選択、Enterで実行
- カスタムコマンド(URL/Command/File)、Bookmark、ローカルアプリケーション、任意のMarkdownファイル内のリンクが対象
- エイリアス + 引数(`g react` → `https://google.com/search?q=react`)
- 軽量で省メモリ。ポータブル運用で、`config.json` / `commands.json` / `logs/` は exe と同じディレクトリに置く

## 設計方針

- シンプルアンドクリーン
- 抽象化は必要なときだけ行う(YAGNI)
- ファイル/関数は1つの責務を持つ(SOLID)

## 技術スタック

- Frontend: React 19, TypeScript, Vite, Fuse.js
- Backend: Rust (Tauri v2)
- Style: Tailwind CSS 3 + DaisyUI v4
- Test: Jest(フロント)、`cargo test`(バックエンド)
- パッケージマネージャ: pnpm

## 構成

- `src/` — フロントエンド。`components/`(UI)、`hooks/`(状態とロジック)、`context/`、`search.ts`(Fuse.js)、`tauri.ts`(IPC呼び出しはここに集約)
- `src-tauri/src/commands/` — コマンドの `domain` / `registry`(メモリ上の一覧) / `store`(commands.json) / `loader`(各ソースの読み込み) / `execution` / `ipc`。`bookmark` / `application` / `markdown` が外部ソース
- `src-tauri/src/config/` — 設定(config.json)
- `src-tauri/src/` 直下 — `lifecycle`(reload・終了)、`window`、`shortcut`、`hotkey_capture`、`tray`、`watcher` など

## 守るべき不変条件

- コマンドの実行は ID で行う(`execute_command(id, args)`)。webview から受け取ったコマンド文字列をそのまま実行しない
- 検索欄に直接入力した URL/パスは `open_target` で開く。URL は許可スキーム(http/https/mailto/ftp)のみ
- URL コマンドの引数はパーセントエンコードする。置換ロジックは Rust(`Command::substitute_args`)と TS(`substituteArgs`、プレビュー用)の2箇所にあるため、変更時は両方を揃える
- `source` は実行時のみ保持し、commands.json には保存しない

## 開発

- 起動: `pnpm tauri:dev`
- push 前に CI と同じチェックを通す: `pnpm tsc --noEmit` / `pnpm lint` / `pnpm format:check` / `pnpm test` / `cargo fmt --check` / `cargo clippy -D warnings` / `cargo test`(いずれも `--manifest-path src-tauri/Cargo.toml` 付きで実行)
- `cargo audit` は週次の別ワークフロー(`.github/workflows/audit.yml`)

## コミット規約

グローバル設定(`~/.claude/CLAUDE.md`)に従う。

## ドキュメント

- 用語集(下記で取り込み済み)。用語は `Avoid` に挙げた言い換えを使わない
- 設計判断の記録は `docs/adr/`。関連する領域を触る前に読む
- Issues は GitHub Issues(`github.com/oni0002/scoot`)

@CONTEXT.md
