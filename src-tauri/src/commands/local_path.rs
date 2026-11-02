use crate::commands::domain::{Command, CATEGORY_FILE, SOURCE_LOCAL};
use crate::settings::domain::LocalPathSettings;
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs::DirEntry;
use std::path::{Path, PathBuf};

/// 全登録フォルダを合わせた件数の上限。設定にはしない。超えたら走査をやめて通知で絞り方を伝える
pub const MAX_ENTRIES: usize = 5000;

/// 上限で打ち切ったときの通知の出所
pub const NOTICE_SOURCE: &str = "localPaths";

#[derive(Default)]
pub struct Scan {
    pub commands: Vec<Command>,
    /// 上限に達して走査を途中でやめたか
    pub truncated: bool,
}

/// どのファイルを一覧にするか。空の `extensions` はファイルなし、`*` はすべて
enum Files {
    None,
    All,
    Extensions(HashSet<String>),
}

impl Files {
    fn from_extensions(extensions: &[String]) -> Self {
        if extensions.is_empty() {
            return Files::None;
        }
        if extensions.iter().any(|e| e == "*") {
            return Files::All;
        }
        Files::Extensions(
            extensions
                .iter()
                .map(|e| e.trim_start_matches('.').to_lowercase())
                .collect(),
        )
    }

    fn includes(&self, path: &Path) -> bool {
        match self {
            Files::None => false,
            Files::All => true,
            Files::Extensions(set) => path
                .extension()
                .is_some_and(|e| set.contains(&e.to_string_lossy().to_lowercase())),
        }
    }
}

/// 登録フォルダ1つ分の、たどる深さと一覧にするもの
struct Rule<'a> {
    depth: usize,
    files: Files,
    include_folders: bool,
    alias: Option<&'a str>,
    alias_only: bool,
}

impl Rule<'_> {
    /// この規則が一覧にした `command` に、検索の入口を足す。登録フォルダが重なって複数の規則が
    /// 同じものを一覧にしたときは、どれかが通常の検索に出すなら通常の検索にも出す
    fn tag(&self, command: &mut Command) {
        command.scoped_only &= self.alias_only;
        if let Some(alias) = self.alias {
            if !command.scopes.iter().any(|s| s == alias) {
                command.scopes.push(alias.to_string());
            }
        }
    }
}

/// 登録フォルダの下のファイルとフォルダを Command にする
pub async fn load(settings: &LocalPathSettings) -> Result<Scan, crate::error::AppError> {
    let settings = settings.clone();
    tokio::task::spawn_blocking(move || scan(&settings, MAX_ENTRIES))
        .await
        .map_err(|e| crate::error::AppError::System(format!("Failed to scan local paths: {}", e)))
}

/// 全登録フォルダをまとめて幅優先でたどる。`cap` で打ち切ったとき、浅いものから残る
fn scan(settings: &LocalPathSettings, cap: usize) -> Scan {
    let excluded: HashSet<String> = settings
        .exclude_names
        .iter()
        .map(|n| n.to_lowercase())
        .collect();
    let rules: Vec<Rule> = settings
        .entries
        .iter()
        .map(|e| Rule {
            depth: e.depth,
            files: Files::from_extensions(&e.extensions),
            include_folders: e.include_folders,
            alias: e.alias.as_deref(),
            alias_only: e.alias_only,
        })
        .collect();

    let mut queue: VecDeque<(PathBuf, usize, &Rule)> = VecDeque::new();
    for (entry, rule) in settings.entries.iter().zip(&rules) {
        let root = expand(&entry.path);
        if root.is_dir() {
            queue.push_back((root, 0, rule));
        } else {
            log::warn!("Local path '{}' is not a folder; skipped.", root.display());
        }
    }

    let mut commands: Vec<Command> = Vec::new();
    // パス → commands の位置
    let mut seen: HashMap<String, usize> = HashMap::new();
    while let Some((dir, level, rule)) = queue.pop_front() {
        let entries = match std::fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(e) => {
                log::debug!("Cannot read '{}': {}", dir.display(), e);
                continue;
            }
        };
        for entry in entries.flatten() {
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            // シンボリックリンクとジャンクションはたどらない(循環や登録外への逸脱を防ぐ)。
            // OneDrive のクラウドファイルも再解析ポイントだが、名前の代理ではないので対象に残る
            let name = entry.file_name().to_string_lossy().to_lowercase();
            // `.git` や `.env` などドット始まりは常に飛ばす(設定にしない)
            if file_type.is_symlink()
                || name.starts_with('.')
                || is_hidden(&entry)
                || excluded.contains(&name)
            {
                continue;
            }
            let path = entry.path();
            let listed = if file_type.is_dir() {
                if level + 1 < rule.depth {
                    queue.push_back((path.clone(), level + 1, rule));
                }
                rule.include_folders
            } else {
                rule.files.includes(&path)
            };
            if !listed {
                continue;
            }
            // 登録フォルダが重なっても、同じパスは1回だけ出す(上限の数え方もそろう)。
            // どの登録の規則で出ても Command は同じなので、どれか1つが一覧にすれば出る。
            // 検索の入口(Alias)は、一覧にした規則ぶんが集まる
            match seen.entry(key_of(&path)) {
                Entry::Occupied(slot) => rule.tag(&mut commands[*slot.get()]),
                Entry::Vacant(slot) => {
                    if commands.len() == cap {
                        return Scan {
                            commands,
                            truncated: true,
                        };
                    }
                    slot.insert(commands.len());
                    let mut command = command_for(&path);
                    command.scoped_only = true;
                    rule.tag(&mut command);
                    commands.push(command);
                }
            }
        }
    }
    Scan {
        commands,
        truncated: false,
    }
}

/// `~` と `%VAR%` を展開し、`/` を Windows の区切りにそろえ、末尾の区切りを落とす
fn expand(path: &str) -> PathBuf {
    Path::new(&crate::os::expand_env_vars(path))
        .components()
        .collect()
}

/// 書き方(`~`・`%VAR%`・区切り・末尾の区切り・大文字小文字)が違っても、同じパスなら同じになるキー
pub fn path_key(path: &str) -> String {
    key_of(&expand(path))
}

/// 重複判定のキー。Windows のパスは大文字小文字を区別しない
fn key_of(path: &Path) -> String {
    let key = path.to_string_lossy();
    if cfg!(windows) {
        key.to_lowercase()
    } else {
        key.into_owned()
    }
}

/// 隠し属性かシステム属性を持つもの
#[cfg(windows)]
fn is_hidden(entry: &DirEntry) -> bool {
    use std::os::windows::fs::MetadataExt;
    const FILE_ATTRIBUTE_HIDDEN: u32 = 0x2;
    const FILE_ATTRIBUTE_SYSTEM: u32 = 0x4;
    // Windows の DirEntry::metadata は列挙時の情報を使うので、追加の I/O はない
    entry
        .metadata()
        .is_ok_and(|m| m.file_attributes() & (FILE_ATTRIBUTE_HIDDEN | FILE_ATTRIBUTE_SYSTEM) != 0)
}

// ドット始まりは呼び出し側で飛ばすので、Windows 以外に隠し属性はない
#[cfg(not(windows))]
fn is_hidden(_entry: &DirEntry) -> bool {
    false
}

/// 名前はファイル名(拡張子つき)、説明は親フォルダのパス
fn command_for(path: &Path) -> Command {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let parent = path
        .parent()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default();
    Command::new(
        name,
        CATEGORY_FILE,
        SOURCE_LOCAL,
        path.to_string_lossy(),
        parent,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::domain::LocalPathEntry;

    /// `entries` を作った一時フォルダ。`/` で終わるものはフォルダ
    fn tree(entries: &[&str]) -> PathBuf {
        let root = std::env::temp_dir().join(format!("scoot-local-{}", nanoid::nanoid!()));
        std::fs::create_dir_all(&root).unwrap();
        for entry in entries {
            let path = root.join(entry);
            if entry.ends_with('/') {
                std::fs::create_dir_all(&path).unwrap();
            } else {
                std::fs::create_dir_all(path.parent().unwrap()).unwrap();
                std::fs::write(&path, "").unwrap();
            }
        }
        root
    }

    /// すべてのファイルとフォルダを、既定の深さで一覧にする登録
    fn entry(path: &Path) -> LocalPathEntry {
        LocalPathEntry {
            path: path.to_string_lossy().into_owned(),
            extensions: vec!["*".to_string()],
            ..LocalPathEntry::default()
        }
    }

    fn settings(root: &Path) -> LocalPathSettings {
        LocalPathSettings {
            enabled: true,
            entries: vec![entry(root)],
            ..LocalPathSettings::default()
        }
    }

    /// 見つかったものを、登録フォルダからの `/` 区切りの相対パスで(順不同なので並べて)
    fn found(scan: &Scan, root: &Path) -> Vec<String> {
        let mut found: Vec<String> = scan
            .commands
            .iter()
            .map(|c| {
                Path::new(&c.command)
                    .strip_prefix(root)
                    .unwrap()
                    .components()
                    .map(|p| p.as_os_str().to_string_lossy().into_owned())
                    .collect::<Vec<_>>()
                    .join("/")
            })
            .collect();
        found.sort();
        found
    }

    fn found_with(settings: &LocalPathSettings, root: &Path) -> Vec<String> {
        found(&scan(settings, MAX_ENTRIES), root)
    }

    #[test]
    fn an_entry_opens_its_path_and_shows_its_folder() {
        let root = tree(&["docs/Guide.PDF"]);
        let scan = scan(&settings(&root), MAX_ENTRIES);
        let file = scan
            .commands
            .iter()
            .find(|c| c.name == "Guide.PDF")
            .unwrap();
        let folder = root.join("docs");
        assert_eq!(file.command, folder.join("Guide.PDF").to_string_lossy());
        assert_eq!(file.description, folder.to_string_lossy());
        assert_eq!(file.category, CATEGORY_FILE);
        assert_eq!(file.source, SOURCE_LOCAL);
        assert!(scan.commands.iter().any(|c| c.name == "docs"));
    }

    #[test]
    fn depth_counts_levels_below_the_registered_folder() {
        let root = tree(&["a.md", "d1/b.md", "d1/d2/c.md", "d1/d2/d3/e.md"]);
        let mut s = settings(&root);
        s.entries[0].depth = 1;
        assert_eq!(found_with(&s, &root), ["a.md", "d1"]);
        s.entries[0].depth = 2;
        assert_eq!(found_with(&s, &root), ["a.md", "d1", "d1/b.md", "d1/d2"]);
        s.entries[0].depth = 3;
        assert_eq!(
            found_with(&s, &root),
            ["a.md", "d1", "d1/b.md", "d1/d2", "d1/d2/c.md", "d1/d2/d3"]
        );
    }

    #[test]
    fn extensions_pick_files_case_insensitively_and_star_takes_all() {
        let root = tree(&["a.MD", "b.pdf", "c.txt", "README"]);
        let mut s = settings(&root);
        s.entries[0].include_folders = false;

        s.entries[0].extensions = vec![];
        assert!(found_with(&s, &root).is_empty());
        s.entries[0].extensions = vec!["md".to_string(), ".PDF".to_string()];
        assert_eq!(found_with(&s, &root), ["a.MD", "b.pdf"]);
        s.entries[0].extensions = vec!["*".to_string()];
        assert_eq!(found_with(&s, &root), ["README", "a.MD", "b.pdf", "c.txt"]);
    }

    #[test]
    fn folders_can_be_left_out_while_their_files_stay() {
        let root = tree(&["p/notes.md", "q/"]);
        let mut s = settings(&root);
        assert_eq!(found_with(&s, &root), ["p", "p/notes.md", "q"]);
        s.entries[0].include_folders = false;
        assert_eq!(found_with(&s, &root), ["p/notes.md"]);
        // ファイルなし・フォルダのみ
        s.entries[0].include_folders = true;
        s.entries[0].extensions = vec![];
        assert_eq!(found_with(&s, &root), ["p", "q"]);
    }

    #[test]
    fn each_entry_walks_to_its_own_depth() {
        let root = tree(&["p/a.md", "p/d/b.md", "q/a.md", "q/d/b.md", "q/d/e/c.md"]);
        let mut shallow = entry(&root.join("p"));
        shallow.depth = 1;
        let mut deeper = entry(&root.join("q"));
        deeper.depth = 2;
        let s = LocalPathSettings {
            entries: vec![shallow, deeper],
            ..settings(&root)
        };
        assert_eq!(
            found_with(&s, &root),
            ["p/a.md", "p/d", "q/a.md", "q/d", "q/d/b.md", "q/d/e"]
        );
    }

    #[test]
    fn each_entry_picks_its_own_files_and_folders() {
        let root = tree(&[
            "docs/a.md",
            "docs/b.pdf",
            "docs/sub/",
            "code/a.md",
            "code/app/",
        ]);
        let mut docs = entry(&root.join("docs"));
        docs.extensions = vec!["md".to_string()];
        docs.include_folders = false;
        let mut code = entry(&root.join("code"));
        code.extensions = vec![];
        let s = LocalPathSettings {
            entries: vec![docs, code],
            ..settings(&root)
        };
        assert_eq!(found_with(&s, &root), ["code/app", "docs/a.md"]);
    }

    #[test]
    fn excluded_names_skip_the_whole_subtree_whatever_the_case() {
        let root = tree(&[
            "node_modules/pkg/index.js",
            "app/Target/debug/out.exe",
            "app/main.rs",
            "Thumbs.db",
        ]);
        let mut s = settings(&root);
        s.exclude_names.push("thumbs.db".to_string());
        assert_eq!(found_with(&s, &root), ["app", "app/main.rs"]);
    }

    #[test]
    fn dot_prefixed_entries_are_skipped_with_their_subtree() {
        let root = tree(&[
            ".config/app.toml",
            ".env",
            "src/.hidden/x.md",
            "src/main.rs",
        ]);
        assert_eq!(found_with(&settings(&root), &root), ["src", "src/main.rs"]);
    }

    #[test]
    fn overlapping_registered_folders_list_each_path_once() {
        let root = tree(&["x/a.md", "x/b.txt", "x/y/", "c.md"]);
        // 外側はフォルダだけ、内側は md とフォルダ。どちらかが一覧にするものが1回ずつ出る
        let mut outer = entry(&root);
        outer.extensions = vec![];
        let mut inner = entry(&root.join("x"));
        inner.extensions = vec!["md".to_string()];
        let s = LocalPathSettings {
            entries: vec![outer, inner],
            ..settings(&root)
        };
        assert_eq!(found_with(&s, &root), ["x", "x/a.md", "x/y"]);
        // 両方が出す x/y は上限にも1回だけ数える
        assert!(!scan(&s, 3).truncated);
    }

    #[test]
    fn the_cap_stops_the_walk_and_says_so() {
        let root = tree(&["a.md", "b.md", "c.md"]);
        let mut s = settings(&root);
        s.entries[0].include_folders = false;

        let full = scan(&s, 3);
        assert_eq!(full.commands.len(), 3);
        assert!(!full.truncated);

        let cut = scan(&s, 2);
        assert_eq!(cut.commands.len(), 2);
        assert!(cut.truncated);
    }

    #[test]
    fn a_truncated_walk_keeps_the_shallow_entries() {
        // 片方の枝を深く下りて上限を埋めず、どちらの枝も浅いものが先に残ること
        let root = tree(&[
            "a/a1.md",
            "a/d/deep1.md",
            "a/d/deep2.md",
            "b/b1.md",
            "b/d/deep3.md",
            "b/d/deep4.md",
        ]);
        let mut s = settings(&root);
        s.entries[0].include_folders = false;
        let scan = scan(&s, 2);
        assert!(scan.truncated);
        assert_eq!(found(&scan, &root), ["a/a1.md", "b/b1.md"]);
    }

    #[test]
    fn entries_with_their_own_rules_share_one_cap() {
        let first = tree(&["a.md", "b.md", "skip.txt"]);
        let second = tree(&["c.txt", "d/"]);
        let mut md_only = entry(&first);
        md_only.extensions = vec!["md".to_string()];
        md_only.include_folders = false;
        let mut s = settings(&second);
        s.entries.insert(0, md_only);
        assert_eq!(scan(&s, MAX_ENTRIES).commands.len(), 4);
        assert!(!scan(&s, 4).truncated);
        assert!(scan(&s, 3).truncated);
    }

    #[test]
    fn a_missing_registered_path_is_skipped() {
        let root = tree(&["a.md"]);
        let mut s = settings(&root);
        s.entries.insert(0, entry(&root.join("missing")));
        assert_eq!(found_with(&s, &root), ["a.md"]);
    }

    /// 見つかったものの(登録フォルダからの相対パス, scopes, scoped_only)。順不同なので並べる
    fn tagged(scan: &Scan, root: &Path) -> Vec<(String, Vec<String>, bool)> {
        let mut tagged: Vec<_> = scan
            .commands
            .iter()
            .map(|c| {
                let relative = Path::new(&c.command).strip_prefix(root).unwrap();
                (
                    relative.to_string_lossy().replace('\\', "/"),
                    c.scopes.clone(),
                    c.scoped_only,
                )
            })
            .collect();
        tagged.sort();
        tagged
    }

    #[test]
    fn an_alias_tags_what_its_folder_lists_and_alias_only_keeps_it_out_of_search() {
        let root = tree(&["p/a.md", "q/b.md"]);
        let mut p = entry(&root.join("p"));
        p.alias = Some("pj".to_string());
        p.alias_only = true;
        let mut q = entry(&root.join("q"));
        q.alias = Some("nt".to_string());
        let s = LocalPathSettings {
            entries: vec![p, q, entry(&root)],
            ..settings(&root)
        };
        // 外側(Alias なし・通常の検索に出す)が p と q の中身も一覧にするので、重なった分は通常の検索にも出る
        let scan = scan(&s, MAX_ENTRIES);
        let tags = tagged(&scan, &root);
        let pick = |name: &str| tags.iter().find(|t| t.0 == name).unwrap();
        assert_eq!(
            pick("p/a.md"),
            &("p/a.md".into(), vec!["pj".to_string()], false)
        );
        assert_eq!(
            pick("q/b.md"),
            &("q/b.md".into(), vec!["nt".to_string()], false)
        );
    }

    #[test]
    fn alias_only_hides_an_entry_only_when_every_folder_listing_it_is_alias_only() {
        let root = tree(&["p/a.md"]);
        let mut inner = entry(&root.join("p"));
        inner.alias = Some("pj".to_string());
        inner.alias_only = true;
        let mut outer = entry(&root);
        outer.alias = Some("all".to_string());
        outer.alias_only = true;
        let both_only = LocalPathSettings {
            entries: vec![inner.clone(), outer.clone()],
            ..settings(&root)
        };
        let scan_only = scan(&both_only, MAX_ENTRIES);
        let a = tagged(&scan_only, &root)
            .into_iter()
            .find(|t| t.0 == "p/a.md")
            .unwrap();
        // 重なった2つの登録の Alias がどちらも入口になる
        assert_eq!(a.1, ["pj", "all"]);
        assert!(a.2);

        outer.alias_only = false;
        let one_open = LocalPathSettings {
            entries: vec![inner, outer],
            ..settings(&root)
        };
        let scan_open = scan(&one_open, MAX_ENTRIES);
        let a = tagged(&scan_open, &root)
            .into_iter()
            .find(|t| t.0 == "p/a.md")
            .unwrap();
        assert!(!a.2);
    }

    #[test]
    fn a_folder_without_an_alias_lists_into_search_as_before() {
        let root = tree(&["a.md"]);
        let scan = scan(&settings(&root), MAX_ENTRIES);
        assert_eq!(tagged(&scan, &root), [("a.md".to_string(), vec![], false)]);
    }

    #[cfg(windows)]
    #[test]
    fn hidden_and_system_entries_are_skipped_with_their_contents() {
        let root = tree(&["shown.md", "hidden.md", "secret/inside.md", "sys.md"]);
        let attrib = |flag: &str, name: &str| {
            let ok = std::process::Command::new("attrib")
                .arg(flag)
                .arg(root.join(name))
                .status()
                .unwrap()
                .success();
            assert!(ok, "attrib {flag} {name}");
        };
        attrib("+h", "hidden.md");
        attrib("+h", "secret");
        attrib("+s", "sys.md");
        assert_eq!(found_with(&settings(&root), &root), ["shown.md"]);
    }

    #[cfg(windows)]
    #[test]
    fn junctions_are_not_followed() {
        let outside = tree(&["secret.md"]);
        let root = tree(&["inside.md"]);
        // ジャンクションは管理者権限なしで作れる。外へ出るものと、自分自身へ戻る循環の2つ
        for (link, target) in [("out", &outside), ("loop", &root)] {
            let made = std::process::Command::new("cmd")
                .arg("/C")
                .arg("mklink")
                .arg("/J")
                .arg(root.join(link))
                .arg(target)
                .output()
                .unwrap();
            if !made.status.success() {
                eprintln!(
                    "skipped: mklink /J failed: {}",
                    String::from_utf8_lossy(&made.stderr)
                );
                return;
            }
        }
        assert_eq!(found_with(&settings(&root), &root), ["inside.md"]);
    }
}
