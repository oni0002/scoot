# Scoot

ホットキーで呼び出すキーボード駆動のコマンドランチャー。ユーザーが登録したコマンドや外部ソースから収集したコマンドをfuzzy searchで探して実行する。

## Language

### コアエンティティ

**Command**:
ユーザーが実行できる単一の操作。名前・カテゴリ・実行内容・任意のエイリアスを持つ。
_Avoid_: Action, item, entry

**Category**:
Command の実行方法を決める4種類の区分。`url`（ブラウザで開く）、`file`（ファイル/フォルダを開く）、`command`（シェルコマンドを実行）、`scoot`（アプリ内部の操作）。
_Avoid_: Type, kind

**Source**:
Command がどこから読み込まれたかの出所区分。`user`（手動登録）、`bookmark`（ブラウザブックマーク）、`application`（OSのショートカットファイル）、`scoot`（組み込みコマンド）、Markdown Link では抽出元のファイル名。実行時のみ保持され、永続化されない。
_Avoid_: Origin, provider

**Scoot Command**:
アプリ自身の操作を行う組み込みの Command（設定を開く、再読み込みする等）。Source が `scoot` であり Category も `scoot`。
_Avoid_: Built-in command, internal command

### 引数とエイリアス

**Alias**:
引数を受け取る Command に付けた短い呼び出しキーワード。ユーザーが `<alias> <args>` と入力すると Argument Mode に入り、引数がコマンドに渡される。
_Avoid_: Prompt, trigger, keyword

**Placeholder**:
Command の実行内容に埋め込む引数の位置指定子。`{$1}` `{$2}` は1番目・2番目の引数、`{$*}` は全引数を展開する。
_Avoid_: Template variable, parameter

**Argument Mode**:
ユーザーが Alias を入力した後に引数を入力している UI の状態。この状態では検索ではなく Placeholder への値の埋め込みが行われる。
_Avoid_: Prompt mode

### 検索

**Fuzzy Search**:
入力クエリをコマンドの名前・Alias に対してあいまい一致で絞り込む検索。スコアで順位付けされる。
_Avoid_: Search, filter

**Direct Open**:
入力クエリが URL・ファイルパスのパターンに一致した場合に、検索結果を経由せず直接開く動作。
_Avoid_: Direct execute, quick open

**Ignored**:
設定によって検索結果から非表示にされた Command の状態。Command 自体は保持されるが表示されない。
_Avoid_: Hidden, excluded, disabled

### 外部ソース

**Bookmark**:
ブラウザ（Chrome/Edge/Brave）から取得した URL Command。起動時・再読み込み時に読み込まれ、Source は `bookmark`。
_Avoid_: Favorite, browser link

**Application**:
OS のスタートメニュー等の指定ディレクトリから収集したショートカットファイル（`.lnk`）由来の Command。Source は `application`。
_Avoid_: App, shortcut

**Markdown Link**:
指定した Markdown ファイルから抽出した URL または ファイルパスの Command。Source は抽出元の Markdown のファイル名（行のバッジに表示される）。
_Avoid_: Link, md link

### 設定・ライフサイクル

**Config**:
アプリの動作設定全体。ホットキー・テーマ・Fuzzy Search の閾値・各外部ソースの有効化と対象パスなどを含む。`config.json` に保存される。
_Avoid_: Settings, preferences

**Hotkey**:
アプリをどこからでも呼び出すグローバルキーボードショートカット。OS レベルで登録される。
_Avoid_: Shortcut (ファイルショートカットと混同するため), keybinding

**Config Screen**:
Config を GUI で編集するための画面。Scoot Command (OpenSettings) または3点メニューから開く。変更は画面を閉じるとき(Esc・戻る)にまとめて保存される。保存に失敗したら画面は閉じず、理由を表示する。`ignored` の削除・各外部ソースの有効化・パス設定・テーマ・Hotkey などを編集できる。
_Avoid_: Settings screen, preferences screen

**Reload**:
Config と全 Command ソースを再読み込みしてアプリの状態を最新化する操作。設定ファイルの変更検知・タイマー・手動操作で発生する。
_Avoid_: Refresh, restart
