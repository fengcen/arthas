
use chrono::*;


/// UTC Date Time.
pub struct UtcDateTime(DateTime<UTC>);

impl UtcDateTime {
    /// Create UTC DateTime from now.
    pub fn new() -> UtcDateTime {
        UtcDateTime(UTC::now())
    }
}
