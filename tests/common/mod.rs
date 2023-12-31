use std::{fs, path::Path};
use walkdir::WalkDir;

#[allow(dead_code)]
pub(crate) fn compare_template(snapshot_path: &Path, template_path: &Path) {
    for entry in WalkDir::new(template_path).into_iter() {
        entry.map_or((), |e| {
            if e.path().is_file() {
                compare(snapshot_path, template_path, e.path());
            }
        })
    }
}

#[allow(dead_code)]
pub(crate) fn compare_template_skip(
    snapshot_path: &Path,
    template_path: &Path,
    skipped_folders: &[&str],
) {
    // https://docs.rs/walkdir/latest/walkdir/struct.IntoIter.html#method.skip_current_dir
    let mut it = WalkDir::new(template_path).into_iter();
    loop {
        let entry = match it.next() {
            None => break,
            Some(entry) => entry.unwrap(),
        };
        let file_name = entry.file_name().to_string_lossy().to_string();
        let skip_entry = skipped_folders.contains(&file_name.as_str());

        if skip_entry && entry.file_type().is_dir() {
            it.skip_current_dir();
            continue;
        }
        if entry.file_type().is_file() && !skipped_folders.contains(&file_name.as_str()) {
            compare(snapshot_path, template_path, entry.path());
        }
    }
}

fn compare(snapshot_path: &Path, path: &Path, entry: &Path) {
    let content = fs::read_to_string(entry).unwrap();
    let name = entry.file_name().and_then(|v| v.to_str());
    insta::with_settings!({
        snapshot_path => snapshot_path
        .join(entry.strip_prefix(path).unwrap())
        .parent()
        .unwrap(),
        prepend_module_to_snapshot => false,
    },{
        insta::assert_snapshot!(name, content);
    });
}
