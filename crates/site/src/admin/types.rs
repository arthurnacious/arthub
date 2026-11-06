//! What the dashboard's server functions send to the browser.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MessageRow {
    pub id: i64,
    /// Unix seconds.
    pub created_at: i64,
    pub name: String,
    pub email: String,
    pub project_type: String,
    pub budget: String,
    pub body: String,
    pub status: String,
    pub replies: i64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReplyRow {
    pub id: i64,
    pub created_at: i64,
    pub body: String,
    /// Set if the email didn't go out.
    pub error: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Thread {
    pub message: MessageRow,
    pub replies: Vec<ReplyRow>,
}

/// One item of a reorderable collection, with its row id.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Row<T> {
    pub id: i64,
    pub data: T,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kind {
    Projects,
    Roles,
    SkillGroups,
}

/// A file uploaded through the dashboard.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Upload {
    /// URL path, e.g. `/media/uploads/ab12.jpg`.
    pub src: String,
    /// The same without the leading slash (the game room's asset path).
    pub asset: String,
    pub width: u32,
    pub height: u32,
    pub video: bool,
}

/// "3 Oct 2026, 14:05" from Unix seconds (UTC).
pub fn when(t: i64) -> String {
    let days = t.div_euclid(86_400);
    let secs = t.rem_euclid(86_400);
    // Howard Hinnant's civil-from-days.
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    format!(
        "{d} {} {y}, {:02}:{:02}",
        MONTHS[(m - 1) as usize],
        secs / 3600,
        secs % 3600 / 60
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn formats_dates() {
        assert_eq!(super::when(0), "1 Jan 1970, 00:00");
        assert_eq!(super::when(1_792_142_100), "16 Oct 2026, 09:15");
    }
}
