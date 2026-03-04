use std::path::{Path, PathBuf};

pub fn chrome_cache_candidates(local_app_data: &Path, profile: &str) -> Vec<PathBuf> {
    let user_data = local_app_data.join("Google").join("Chrome").join("User Data");
    let profile_root = user_data.join(profile);

    vec![
        profile_root.join("Cache"),
        profile_root.join("Network").join("Cache"),
    ]
}

pub fn existing_chrome_cache_dirs(local_app_data: &Path, profile: &str) -> Vec<PathBuf> {
    chrome_cache_candidates(local_app_data, profile)
        .into_iter()
        .filter(|path| path.exists() && path.is_dir())
        .collect()
}