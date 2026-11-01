use crate::commands::domain::{Command, CATEGORY_FILE, SOURCE_LOCAL};
use crate::settings::domain::LocalPathSettings;
use std::collections::{HashSet, VecDeque};
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

/// 登録フォルダの下のファイルとフォルダを Command にする
pub async fn load(settings: &LocalPathSettings) -> Result<Scan, crate::error::AppError> {
    let settings = settings.clone();
    tokio::task::spawn_blocking(move || scan(&settings, MAX_ENTRIES))
        .await
        .map_err(|e| crate::error::AppError::System(format!("Failed to scan local paths: {}", e)))
}

/// 全登録フォルダをまとめて幅優先でたどる。`cap` で打ち切ったとき、浅いものから残る
fn scan(settings: &LocalPathSettings, cap: usize) -> Scan {
    let files = Files::from_extensions(&settings.extensions);
    let excluded: HashSet<String> = settings
        .exclude_names
        .iter()
        .map(|n| n.to_lowercase())
        .collect();

    let mut queue: VecDeque<(PathBuf, usize)> = VecDeque::new();
    for path in &settings.paths {
        // `~/docs` の展開で混ざる `/` を、Windows の区切りにそろえる
        let root: PathBuf = Path::new(&crate::os::expand_env_vars(path))
            .components()
            .collect();
        if root.is_dir() {
            queue.push_back((root, 0));
        } else {
            log::warn!("Local path '{}' is not a folder; skipped.", root.display());
        }
    }

    let mut commands = Vec::new();
    while let Some((dir, level)) = queue.pop_front() {
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
                if level + 1 < settings.depth {
                    queue.push_back((path.clone(), level + 1));
                }
                settings.include_folders
            } else {
                files.includes(&path)
            };
            if !listed {
                continue;
            }
            if commands.len() == cap {
                return Scan {
                    commands,
                    truncated: true,
                };
            }
            commands.push(command_for(&path));
        }
    }
    Scan {
        commands,
        truncated: false,
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

    fn settings(root: &Path) -> LocalPathSettings {
        LocalPathSettings {
            enabled: true,
            paths: vec![root.to_string_lossy().into_owned()],
            extensions: vec!["*".to_string()],
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
        s.depth = 1;
        assert_eq!(found_with(&s, &root), ["a.md", "d1"]);
        s.depth = 2;
        assert_eq!(found_with(&s, &root), ["a.md", "d1", "d1/b.md", "d1/d2"]);
        s.depth = 3;
        assert_eq!(
            found_with(&s, &root),
            ["a.md", "d1", "d1/b.md", "d1/d2", "d1/d2/c.md", "d1/d2/d3"]
        );
    }

    #[test]
    fn extensions_pick_files_case_insensitively_and_star_takes_all() {
        let root = tree(&["a.MD", "b.pdf", "c.txt", "README"]);
        let mut s = settings(&root);
        s.include_folders = false;

        s.extensions = vec![];
        assert!(found_with(&s, &root).is_empty());
        s.extensions = vec!["md".to_string(), ".PDF".to_string()];
        assert_eq!(found_with(&s, &root), ["a.MD", "b.pdf"]);
        s.extensions = vec!["*".to_string()];
        assert_eq!(found_with(&s, &root), ["README", "a.MD", "b.pdf", "c.txt"]);
    }

    #[test]
    fn folders_can_be_left_out_while_their_files_stay() {
        let root = tree(&["p/notes.md", "q/"]);
        let mut s = settings(&root);
        assert_eq!(found_with(&s, &root), ["p", "p/notes.md", "q"]);
        s.include_folders = false;
        assert_eq!(found_with(&s, &root), ["p/notes.md"]);
        // ファイルなし・フォルダのみ
        s.include_folders = true;
        s.extensions = vec![];
        assert_eq!(found_with(&s, &root), ["p", "q"]);
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
    fn the_cap_stops_the_walk_and_says_so() {
        let root = tree(&["a.md", "b.md", "c.md"]);
        let mut s = settings(&root);
        s.include_folders = false;

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
        s.include_folders = false;
        let scan = scan(&s, 2);
        assert!(scan.truncated);
        assert_eq!(found(&scan, &root), ["a/a1.md", "b/b1.md"]);
    }

    #[test]
    fn paths_across_registered_folders_share_one_cap() {
        let first = tree(&["a.md", "b.md"]);
        let second = tree(&["c.md", "d.md"]);
        let mut s = settings(&first);
        s.paths.push(second.to_string_lossy().into_owned());
        s.include_folders = false;
        assert_eq!(scan(&s, MAX_ENTRIES).commands.len(), 4);
        assert!(scan(&s, 3).truncated);
    }

    #[test]
    fn a_missing_registered_path_is_skipped() {
        let root = tree(&["a.md"]);
        let mut s = settings(&root);
        s.paths
            .insert(0, root.join("missing").to_string_lossy().into_owned());
        assert_eq!(found_with(&s, &root), ["a.md"]);
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
