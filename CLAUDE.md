# Scoot

Windows 向けのシンプルなコマンドランチャーアプリ。

- ホットキー(デフォルト `Alt+Space`)で呼び出し
- キーワードでfuzzy search、Tab/Shift-Tabで選択、Enterで実行
- カスタムコマンド(URL/Command/File)、Bookmark、ローカルアプリケーション、任意のMarkdownファイル内のリンクが対象
- エイリアス + 引数(`g react` → `https://google.com/search?q=react`)
- 軽量で省メモリ。ポータブル運用で、`config.json` / `commands.json` / `usage.json`(実行履歴) / `logs/` は exe と同じディレクトリに置く

## 設計方針

- シンプルアンドクリーン
- 抽象化は必要なときだけ行う(YAGNI)
- ファイル/関数は1つの責務を持つ(SOLID)

## 技術スタック

- Frontend: React 19, TypeScript, Vite, Fuse.js
- Backend: Rust (Tauri v2)
- Style: Tailwind CSS 4 + daisyUI 5
- Test: Jest(フロント)、`cargo test`(バックエンド)
- パッケージマネージャ: pnpm

## 構成

- `src/` — フロントエンド。`components/`(UI)、`hooks/`(状態とロジック)、`context/`、`search.ts`(Fuse.js と、略称を拾う `boundaryMatch.ts` の合成)、`tauri.ts`(IPC呼び出しはここに集約)、`listen.ts`(バックエンドのイベント購読。解除が購読完了より先でも漏れない)
- `src-tauri/src/commands/` — コマンドの `domain` / `registry`(メモリ上の一覧) / `store`(commands.json) / `loader`(各ソースの読み込み) / `execution` / `ipc`。`bookmark` / `application` / `markdown` が外部ソース
- `src-tauri/src/config/` — 設定(config.json)
- `src-tauri/src/` 直下 — `lifecycle`(reload・終了)、`window`、`shortcut`、`hotkey_capture`、`tray`、`system_theme`(タスクバーのテーマ。トレイアイコンの切り替え用)、`watcher`(ファイルの変更を `request-reload` にするだけ。まとめて1回の reload にするのは `lifecycle`)、`json`(JSON の読み込みと一時ファイル経由の書き込み。config / commands / usage の保存はここを通す) など
- `design/logo/generate.mjs` — ロゴの SVG、`src-tauri/icons/` 一式(32px 以下は小サイズ用の形、トレイ用の単色アイコン)、UI が描くパス(`src/logo.ts`)を生成する。アイコンは手で編集せず、このスクリプトの寸法を変えて再生成する

## 守るべき不変条件

- コマンドの実行は ID で行う(`execute_command(id, args)`)。webview から受け取ったコマンド文字列をそのまま実行しない
- 検索欄に直接入力した URL/パスは `open_target` で開く。URL は許可スキーム(http/https/mailto/ftp)のみ
- URL コマンドの引数はパーセントエンコードする。置換ロジックは Rust(`Command::substitute_args`)と TS(`substituteArgs`、プレビュー用)の2箇所にあるため、変更時は両方を揃える。両方が `src/__tests__/fixtures/substitute-cases.json` の同じケースを検証する
- `source` は実行時のみ保持し、commands.json には保存しない
- 外部ソースのコマンドに紐づく設定(`ignored` / `aliases`)は `command` 文字列をキーに config.json へ保存する。外部コマンドの ID は `DefaultHasher` のハッシュで、Rust のバージョンをまたいで同じ値になる保証がないので、永続化には使わない(`docs/adr/0003`)
- 実行履歴(`usage.json`)は検索結果の並べ替えにだけ使い、キーは `command` 文字列(ID は外部コマンドで安定しないので使わない)。順位は `fuseScore - boost`(小さいほど上)で、`boost = 0.05 * w / (w + 3)`、`w = 回数 * 0.5^(最終使用からの日数 / 30)`。上限があるので、履歴だけで良く一致する候補を逆転させない。式は `src/search.ts` にあるので、変えるときは `search.test.ts` と揃える

## UI

- スタイルは Tailwind 4 + daisyUI 5。設定は `tailwind.config.js` ではなく `src/App.css`(`@plugin "daisyui"`)に書く。テーマは自前の `scoot-light` / `scoot-dark` の2つだけ(daisyUI 組み込みテーマは `themes: false` で無効)。設定値は `system` / `light` / `dark` で、`src/theme.ts` の `THEME_OPTIONS` と `src-tauri/src/config/domain.rs` の `THEMES` の値の一覧を揃える。色を変えたら、全画面のコントラスト(AA 4.5:1)を測り直す。文字を薄くするのは `text-muted`(85%)と `text-subtle`(70%、行の説明用)の2段だけ。未知のテーマ名は読み込み時に既定へ丸める
- daisyUI 4 の `form-control` / `label-text` / `input-bordered` は v5 で廃止。フォームのラベルは Tailwind のユーティリティで組む
- Tailwind 4 の `space-y-*` は、最初の要素の上余白(`margin-top`)を 0 にする。先頭に余白が要る要素(`.divider` など)を置くときは、コンテナに `pt-*` を持たせる
- 新しいクラスを使うときは、v4 の記憶ではなく v5 のドキュメントで存在を確認する

## 開発

- 起動: `pnpm tauri:dev`
- ブラウザでの UI 確認: `pnpm dev` を起動して `http://localhost:1420` を開くと、`src/dev/mockBackend.ts` のモックで動く(開発時のみ・Tauri の外でのみ有効、本番ビルドには入らない)。ウィンドウ特性(透過・ホットキー・WebView2)は再現しないので、そこは `pnpm tauri:dev` で確認する。`TauriAPI` にコマンドを足したら、モックにも足す(テストが漏れを検出する)
- push 前に CI と同じチェックを通す: `pnpm tsc --noEmit` / `pnpm lint` / `pnpm format:check` / `pnpm test` / `cargo fmt --check` / `cargo clippy -D warnings` / `cargo test`(いずれも `--manifest-path src-tauri/Cargo.toml` 付きで実行)
- `cargo audit` は週次の別ワークフロー(`.github/workflows/audit.yml`)

## コミット規約

グローバル設定(`~/.claude/CLAUDE.md`)に従う。

## ドキュメント

- 用語集(下記で取り込み済み)。用語は `Avoid` に挙げた言い換えを使わない
- 設計判断の記録は `docs/adr/`。関連する領域を触る前に読む
- Issues は GitHub Issues(`github.com/oni0002/scoot`)

@CONTEXT.md
