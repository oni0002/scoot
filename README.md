# Scoot

シンプルで高速なWindows用コマンドランチャー

- **グローバルキー** - `Alt+Space`で起動
- **あいまい検索** - Fuse.jsによるfuzzy search
- **アプリケーション** - スタートアップに登録されているアプリケーションの検索
- **ブックマーク** - ブラウザのブックマークの検索
- **カスタムショートカット** - ファイルパスやURL、コマンドを登録できる
- **プロンプトモード** - `g react` --> `https://google.com/search?q=react`のように、プロンプト+引数を展開できる

## Usage

### 基本操作

1. **起動** - `Alt + Space` でウィンドウを表示
2. **検索** - キーワードを入力して検索
3. **選択** - `Tab`/`Shift+Tab` (または`↑`/`↓`, `Ctrl+N`/`Ctrl+P`) で候補を選択
4. **実行** - `Enter` で選択したショートカットを実行

### キーバインド一覧

| キー操作 | 動作 |
|---|---|
| `Alt + Space` | ウィンドウの表示/非表示 (グローバル) |
| `Esc` | 入力クリア / ウィンドウを閉じる |
| `Enter` | 選択項目の実行 |
| `↑` / `↓` | 検索候補の選択移動 |
| `Tab` / `Shift + Tab` | 検索候補の選択移動 (次へ/前へ) |
| `Ctrl + N` / `P` | 検索候補の選択移動 (Emacsライク) |

## Configuration

### 設定項目

設定はアプリケーションフォルダ配下の`config.json`で定義できる。
直接開く他に、`Open config.json`ショートカットを実行するか、検索バー右の3点メニューから`Open config.json`を選択しても開くことができる。

```json
{
  // 検索結果の数
  "max_results": 30,
  // あいまい検索のしきい値 0(ゆるい)~1(厳しい)
  "fuzzy_threshold": 0.4,
  // ブックマーク検索の設定
  "bookmarks": {
    // ブックマーク検索の有効化
    "enabled": true,
    // 対象のブラウザ。chrome/brave/edge
    "browser": "brave",
  },
  // アプリケーション検索の設定
  "applications": {
    // アプリケーション検索の有効化
    "enabled": true,
    // スキャン対象のディレクトリ。配下の.lnkファイルが検索対象になる
    "directories": [
      "%APPDATA%\\Microsoft\\Windows\\Start Menu\\Programs",
      "C:\\ProgramData\\Microsoft\\Windows\\Start Menu\\Programs"
    ]
  },
  // カラーテーマ。
  "theme": "dark"
}
```

`config.json`は.exeと同じディレクトリ、または`%APPDATA%\oni.scoot\`に配置する。

## ショートカット

### 追加方法

1. `Add Command`、または3点ボタンからAdd commandを選択
2. ダイアログに必要な情報を入力
    - **Name** - 表示名 (例: `Google Search`)
    - **Description** - 説明 (例: `Googleで検索します`)
    - **Command** - 実行したい内容
        - URL - `https://...`
        - ファイルパス - `C:\Path\To\File.exe`
        - シェルコマンド - `npm start` など
    - **Prompt** - (任意) 短縮コマンド。Commandに引数を指定した場合は設定必須 (例: `g`)

### 引数の展開

commandには引数を設定できる。

- `{$n}` - nは数字に置き換え。n番目の引数を展開
- `{$*}` - 引数をすべて展開

> [!note] 例
> 
> - Prompt: `g`
> - Command: `https://www.google.com/search?q={$*}`
> - 実行: `g react hooks`
> - 結果: `https://www.google.com/search?q=react hooks` をブラウザで開く

### カテゴリの種類

- URL - デフォルトのブラウザでURLを開く
- File - Explorerまたはデフォルトのアプリでファイル/ディレクトリを開く
- Command - Powershellでコマンドを実行
- Custom - 任意の実行ファイルを実行

## ライセンス
[License Name Here]