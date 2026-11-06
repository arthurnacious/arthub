//! The contact form's rules, shared by the browser (instant feedback) and the
//! server (the only check that counts). With the `server` feature, also spam
//! protection ([`spam`]) and delivery ([`mail`]).

#[cfg(feature = "server")]
pub mod mail;
#[cfg(feature = "server")]
pub mod spam;

/// Choices for "What are you after?". The server rejects anything else.
pub const PROJECT_TYPES: &[&str] = &[
    "Website",
    "Web app",
    "Systems / backend",
    "3D / interactive",
    "Something else",
];

/// Choices for budget (South African rand).
pub const BUDGETS: &[&str] = &[
    "Under R25k",
    "R25k – R75k",
    "R75k – R150k",
    "R150k+",
    "Not sure yet",
];

pub const NAME_MAX: usize = 100;
pub const MESSAGE_MIN: usize = 10;
pub const MESSAGE_MAX: usize = 5000;
pub const EMAIL_MAX: usize = 254;

/// Form field names, as used in the HTML form and the JSON error map.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Field {
    Name,
    Email,
    ProjectType,
    Budget,
    Message,
}

impl Field {
    pub const ALL: [Field; 5] = [
        Field::Name,
        Field::Email,
        Field::ProjectType,
        Field::Budget,
        Field::Message,
    ];

    pub fn key(self) -> &'static str {
        match self {
            Field::Name => "name",
            Field::Email => "email",
            Field::ProjectType => "project_type",
            Field::Budget => "budget",
            Field::Message => "message",
        }
    }
}

/// The honeypot input's name. Humans never see it; naive bots fill it in.
pub const HONEYPOT: &str = "website";

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Submission {
    pub name: String,
    pub email: String,
    pub project_type: String,
    pub budget: String,
    pub message: String,
}

impl Submission {
    /// Build from form fields (`application/x-www-form-urlencoded` pairs).
    pub fn from_pairs<'a>(pairs: impl IntoIterator<Item = (&'a str, &'a str)>) -> Self {
        let mut s = Self::default();
        for (k, v) in pairs {
            let v = v.to_string();
            match k {
                "name" => s.name = v,
                "email" => s.email = v,
                "project_type" => s.project_type = v,
                "budget" => s.budget = v,
                "message" => s.message = v,
                _ => {}
            }
        }
        s
    }

    /// Trimmed copy (what actually gets validated and sent).
    pub fn normalized(&self) -> Self {
        Self {
            name: self.name.trim().to_string(),
            email: self.email.trim().to_string(),
            project_type: self.project_type.trim().to_string(),
            budget: self.budget.trim().to_string(),
            message: self.message.trim().replace("\r\n", "\n"),
        }
    }
}

/// Validate a submission. All failures at once, one message per field.
pub fn validate(s: &Submission) -> Result<Submission, Vec<(Field, &'static str)>> {
    let s = s.normalized();
    let mut errors = Vec::new();

    let name_len = s.name.chars().count();
    if name_len == 0 {
        errors.push((Field::Name, "Tell me what to call you."));
    } else if name_len > NAME_MAX {
        errors.push((
            Field::Name,
            "That name's over 100 characters. A short version will do.",
        ));
    } else if s.name.chars().any(|c| c.is_control()) {
        errors.push((Field::Name, "That name has characters I can't use."));
    }

    if s.email.is_empty() {
        errors.push((Field::Email, "I need an email to reply to."));
    } else if !is_email(&s.email) {
        errors.push((Field::Email, "That doesn't look like an email address."));
    }

    if !PROJECT_TYPES.contains(&s.project_type.as_str()) {
        errors.push((Field::ProjectType, "Pick what you're after."));
    }
    if !BUDGETS.contains(&s.budget.as_str()) {
        errors.push((
            Field::Budget,
            "Pick a budget range (\"Not sure yet\" is fine).",
        ));
    }

    let msg_len = s.message.chars().count();
    if msg_len < MESSAGE_MIN {
        errors.push((
            Field::Message,
            "Give me a little more to go on (10 characters at least).",
        ));
    } else if msg_len > MESSAGE_MAX {
        errors.push((
            Field::Message,
            "That's over 5,000 characters. Trim it and we'll talk details later.",
        ));
    }

    if errors.is_empty() {
        Ok(s)
    } else {
        Err(errors)
    }
}

/// A pragmatic address check: one `@`, a sane local part, a dotted domain
/// with a 2+ letter TLD, no spaces or angle brackets. (Deliverability is
/// what really validates an address; this just catches typos.)
pub fn is_email(e: &str) -> bool {
    if e.len() > EMAIL_MAX
        || e.chars().any(|c| {
            c.is_whitespace() || c.is_control() || matches!(c, '<' | '>' | ',' | ';' | '"')
        })
    {
        return false;
    }
    let Some((local, domain)) = e.rsplit_once('@') else {
        return false;
    };
    if local.is_empty()
        || local.len() > 64
        || local.contains('@')
        || local.starts_with('.')
        || local.ends_with('.')
    {
        return false;
    }
    let labels: Vec<&str> = domain.split('.').collect();
    labels.len() >= 2
        && labels.iter().all(|l| {
            !l.is_empty()
                && l.len() <= 63
                && !l.starts_with('-')
                && !l.ends_with('-')
                && l.chars().all(|c| c.is_alphanumeric() || c == '-')
        })
        && labels
            .last()
            .is_some_and(|t| t.chars().count() >= 2 && t.chars().all(char::is_alphabetic))
}

/// The email we send: subject and plain-text body.
pub fn compose(s: &Submission) -> (String, String) {
    let subject = format!("Arthub enquiry: {} from {}", s.project_type, s.name);
    let body = format!(
        "New enquiry from the Arthub contact form.\n\n\
         Name:          {}\n\
         Email:         {}\n\
         Project type:  {}\n\
         Budget:        {}\n\n\
         Message:\n\
         {}\n\n\
         --\nReply to this email to answer {} directly.\n",
        s.name, s.email, s.project_type, s.budget, s.message, s.name
    );
    (subject, body)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn good() -> Submission {
        Submission {
            name: "Thandi".into(),
            email: "thandi@example.co.za".into(),
            project_type: PROJECT_TYPES[0].into(),
            budget: BUDGETS[1].into(),
            message: "I need a portfolio site with a pool table in it.".into(),
        }
    }

    fn fields(r: Result<Submission, Vec<(Field, &'static str)>>) -> Vec<Field> {
        r.err()
            .unwrap_or_default()
            .into_iter()
            .map(|(f, _)| f)
            .collect()
    }

    #[test]
    fn accepts_a_good_submission_and_trims_it() {
        let mut s = good();
        s.name = "  Thandi  ".into();
        s.message = "  Hello there, let's talk.\r\nSoon.  ".into();
        let ok = validate(&s).expect("valid");
        assert_eq!(ok.name, "Thandi");
        assert_eq!(ok.message, "Hello there, let's talk.\nSoon.");
    }

    #[test]
    fn name_bounds() {
        let mut s = good();
        s.name = "   ".into();
        assert_eq!(fields(validate(&s)), [Field::Name]);
        s.name = "x".repeat(100);
        assert!(validate(&s).is_ok());
        s.name = "x".repeat(101);
        assert_eq!(fields(validate(&s)), [Field::Name]);
        // Counted in characters, not bytes.
        s.name = "é".repeat(100);
        assert!(validate(&s).is_ok());
    }

    #[test]
    fn email_shapes() {
        for ok in ["a@b.co", "first.last+tag@sub.example.org", "x@münchen.de"] {
            assert!(is_email(ok), "{ok}");
        }
        for bad in [
            "",
            "plain",
            "@example.com",
            "a@",
            "a@b",
            "a@b.c",
            "a b@c.com",
            "a@-b.com",
            "<a@b.com>",
            "a@b.com,c@d.com",
            ".a@b.com",
            "a@b..com",
        ] {
            assert!(!is_email(bad), "{bad}");
        }
    }

    #[test]
    fn choices_must_come_from_the_lists() {
        let mut s = good();
        s.project_type = "Crypto".into();
        s.budget = "a million".into();
        assert_eq!(fields(validate(&s)), [Field::ProjectType, Field::Budget]);
    }

    #[test]
    fn message_bounds() {
        let mut s = good();
        s.message = "too short".into();
        assert_eq!(fields(validate(&s)), [Field::Message]);
        s.message = "x".repeat(10);
        assert!(validate(&s).is_ok());
        s.message = "x".repeat(5000);
        assert!(validate(&s).is_ok());
        s.message = "x".repeat(5001);
        assert_eq!(fields(validate(&s)), [Field::Message]);
    }

    #[test]
    fn reports_every_bad_field_at_once() {
        let errs = fields(validate(&Submission::default()));
        assert_eq!(errs, Field::ALL);
    }

    #[test]
    fn email_subject_and_body() {
        let (subject, body) = compose(&good());
        assert_eq!(subject, "Arthub enquiry: Website from Thandi");
        for part in [
            "Thandi",
            "thandi@example.co.za",
            "Website",
            BUDGETS[1],
            "pool table",
        ] {
            assert!(body.contains(part), "{part}");
        }
    }

    #[test]
    fn parses_form_pairs() {
        let s = Submission::from_pairs([
            ("name", "A"),
            ("email", "a@b.co"),
            ("ignored", "x"),
            ("message", "hi"),
        ]);
        assert_eq!(
            (s.name.as_str(), s.email.as_str(), s.message.as_str()),
            ("A", "a@b.co", "hi")
        );
    }
}
