//! Compile `content/` into the site. A malformed file or a missing image stops
//! the build with a list of every problem (see CONTENT.md).

use content::load::{Paths, load, report, watched};

fn main() {
    // The footer's copyright year. Rebuilt whenever content changes, which
    // is often enough for a year.
    let days = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() / 86_400)
        .unwrap_or(0) as i64;
    println!("cargo::rustc-env=ARTHUB_BUILD_YEAR={}", year_of(days));
    let paths = Paths::from_crate_dir(&std::env::var("CARGO_MANIFEST_DIR").unwrap());
    for p in watched(&paths) {
        println!("cargo::rerun-if-changed={}", p.display());
    }
    match load(&paths) {
        Ok(site) => {
            let out = std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).join("content.rs");
            std::fs::write(out, content::codegen::render(&site)).expect("write generated content");
        }
        Err(errors) => {
            for line in report(&errors).lines() {
                println!("cargo::error={line}");
            }
        }
    }
}

/// Civil year for a day count since 1970-01-01 (Howard Hinnant's algorithm).
fn year_of(days: i64) -> i64 {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    yoe + era * 400 + i64::from(month <= 2)
}
