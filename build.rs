fn main() {
    let dir = std::path::Path::new("apps/web/dist");
    if !dir.exists() {
        std::fs::create_dir_all(dir).expect("failed to create apps/web/dist/");
    }
}
