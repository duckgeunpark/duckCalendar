pub mod oauth;
pub mod sync;

/// keyring service / account identifiers for the stored OAuth tokens.
pub const KEYRING_SERVICE: &str = "com.duckcalendar.app";
pub const KEYRING_ACCOUNT: &str = "google_oauth";

/// Minimal scope: create/read calendar events only (design 최소 권한 원칙).
pub const SCOPE: &str = "https://www.googleapis.com/auth/calendar.events";

pub const AUTH_ENDPOINT: &str = "https://accounts.google.com/o/oauth2/v2/auth";
pub const TOKEN_ENDPOINT: &str = "https://oauth2.googleapis.com/token";
pub const EVENTS_ENDPOINT: &str =
    "https://www.googleapis.com/calendar/v3/calendars/primary/events";
