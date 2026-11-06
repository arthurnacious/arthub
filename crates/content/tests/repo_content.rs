//! The repo's own content/ must always load, and bad content must be reported.
#![cfg(feature = "parse")]

use content::load::{Paths, load};

#[test]
fn repo_content_loads() {
    let site = load(&Paths::from_crate_dir(env!("CARGO_MANIFEST_DIR")))
        .unwrap_or_else(|e| panic!("{e:#?}"));
    assert!(!site.projects.is_empty());
    assert!(
        site.projects.windows(2).all(|w| w[0].year >= w[1].year),
        "newest first"
    );
    for p in &site.projects {
        assert!(
            p.cover.width > 0 && p.cover.height > 0,
            "{} cover has a size",
            p.slug
        );
        assert!(p.cover_asset.starts_with("media/work/"));
    }
    // The generated Rust is what the site compiles; it must at least be non-trivial.
    assert!(content::codegen::render(&site).contains("pub static SITE"));
}

#[test]
fn broken_project_is_reported_with_its_file() {
    let dir = std::env::temp_dir().join(format!("arthub-content-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("content/projects")).unwrap();
    std::fs::create_dir_all(dir.join("assets")).unwrap();
    std::fs::write(
        dir.join("content/projects/x.md"),
        "+++\ntitle = \"X\"\nslug = \"x\"\n+++\n",
    )
    .unwrap();
    let paths = Paths {
        root: dir.clone(),
        content: dir.join("content"),
        assets: dir.join("assets"),
    };
    let errors = load(&paths).unwrap_err();
    assert!(
        errors
            .iter()
            .any(|e| e.starts_with("content/projects/x.md: ") && e.contains("missing field")),
        "{errors:#?}"
    );
    assert!(
        errors
            .iter()
            .any(|e| e.starts_with("content/experience.toml")),
        "{errors:#?}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}
