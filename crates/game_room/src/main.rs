//! Native build of the room. Featured projects come straight from `content/`.

use content::load::{Paths, load, report};

fn main() {
    let mode = match std::env::args().nth(1).as_deref() {
        Some("preview") => bridge::Mode::Preview,
        _ => bridge::Mode::Full,
    };
    let site = match load(&Paths::from_crate_dir(env!("CARGO_MANIFEST_DIR"))) {
        Ok(site) => site,
        Err(errors) => {
            eprintln!("{}", report(&errors));
            std::process::exit(1);
        }
    };
    let featured = site
        .projects
        .into_iter()
        .filter(|p| p.featured)
        .map(|p| bridge::FeaturedProject {
            slug: p.slug,
            title: p.title,
            color: p.accent,
            cover: Some(p.cover_asset),
        })
        .collect();
    game_room::run(mode, featured);
}
