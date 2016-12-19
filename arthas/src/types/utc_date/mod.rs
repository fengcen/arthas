
use chrono::*;


/// UTC Date.
pub struct UtcDate(Date<UTC>);

impl UtcDate {
    /// Create UTC Date from now.
    pub fn new() -> UtcDate {
        UtcDate(UTC::today())
    }
}
