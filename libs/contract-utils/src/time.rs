use crate::env_exports::Timestamp;

pub const SECOND: Timestamp = 1000;
pub const MINUTE: Timestamp = 60 * SECOND;
pub const HOUR: Timestamp = 60 * MINUTE;
pub const DAY: Timestamp = 24 * HOUR;
pub const WEEK: Timestamp = 7 * DAY;
pub const MONTH: Timestamp = 30 * DAY;
pub const YEAR: Timestamp = 365 * DAY;
