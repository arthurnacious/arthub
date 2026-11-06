//! Send one real test enquiry using the configured provider:
//! `just test-email` (reads `.env`). Exits non-zero if delivery fails.

use contact::mail::{MailConfig, Mailer};
use contact::{BUDGETS, PROJECT_TYPES, Submission};

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt().init();
    let cfg = match MailConfig::from_env(true) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Not configured: {e}");
            std::process::exit(2);
        }
    };
    println!("Sending a test enquiry via {} …", cfg.describe());
    let mailer = Mailer::new(cfg.clone()).expect("mail transport");
    let s = Submission {
        name: "Arthub test".into(),
        email: cfg.to.split('<').next_back().unwrap_or(&cfg.to).trim_end_matches('>').trim().to_string(),
        project_type: PROJECT_TYPES[0].into(),
        budget: BUDGETS[4].into(),
        message: "This is a test from `just test-email`. If you can read this, the contact form delivers. Hit reply to check Reply-To.".into(),
    };
    match mailer.send(&s).await {
        Ok(()) => println!("Sent. Check {} (and spam, the first time).", cfg.to),
        Err(e) => {
            eprintln!("Failed: {e}");
            std::process::exit(1);
        }
    }
}
