// Simple Unicode icon constants for better compatibility
pub struct NeonIcons;

impl NeonIcons {
    // Navigation icons - using Unicode arrow symbols
    pub const ARROW_LEFT: &'static str = "←";
    pub const ARROW_RIGHT: &'static str = "→";
    pub const ARROW_CLOCKWISE: &'static str = "↻";
    pub const HOUSE: &'static str = "⌂";
    
    // Security/Connection icons - using Unicode symbols
    pub const LOCK: &'static str = "🔒";
    pub const SHIELD_CHECK: &'static str = "🛡";
    pub const GLOBE: &'static str = "🌐";
    pub const GLOBE_SIMPLE: &'static str = "○";
    
    // UI icons - using clean Unicode symbols
    pub const MAGNIFYING_GLASS: &'static str = "🔍";
    pub const BOOKMARKS: &'static str = "📑";
    pub const BOOKMARK: &'static str = "🔖";
    pub const TRASH: &'static str = "🗑";
    pub const GEAR: &'static str = "⚙";
    pub const TERMINAL: &'static str = "⌨";
    pub const TERMINAL_WINDOW: &'static str = "▣";
    
    // Content/Error icons - using clear symbols
    pub const WARNING: &'static str = "⚠";
    pub const INFO: &'static str = "ℹ";
    pub const CHECK_CIRCLE: &'static str = "✓";
    pub const X_CIRCLE: &'static str = "✗";
    pub const WIFI_SLASH: &'static str = "📵";
    pub const NETWORK_SLASH: &'static str = "🚫";
    pub const PROHIBIT: &'static str = "🚫";
    pub const ARROWS_CLOCKWISE: &'static str = "⟲";
    pub const CERTIFICATE: &'static str = "📜";
    pub const CALENDAR_X: &'static str = "📅";
    pub const LOCK_SIMPLE: &'static str = "🔐";
    pub const FIRE: &'static str = "🔥";
    pub const PACKAGE: &'static str = "📦";
    
    // Action icons - using simple symbols
    pub const PLAY: &'static str = "▶";
    pub const PAUSE: &'static str = "⏸";
    pub const PLUS: &'static str = "+";
    pub const MINUS: &'static str = "-";
    pub const X: &'static str = "×";
    pub const CROSS: &'static str = "✗";
    pub const WRENCH: &'static str = "🔧";
    pub const REFRESH: &'static str = "↻";
    pub const DOWNLOAD: &'static str = "⬇";
    pub const UPLOAD: &'static str = "⬆";
    pub const SEARCH: &'static str = "🔍";
    pub const FOLDER: &'static str = "📁";
    pub const CLOCK: &'static str = "🕐";
    pub const CODE: &'static str = "⌨";
    pub const CHECK: &'static str = "✓";
    pub const HEART: &'static str = "♡";
    pub const STAR: &'static str = "★";
    pub const DELETE: &'static str = "🗑";
    pub const FOLDER_OPEN: &'static str = "📂";
}

impl NeonIcons {
    /// Get icon for URL security status
    pub fn security_icon_for_url(url: &str) -> &'static str {
        if url.starts_with("https://") {
            Self::LOCK
        } else if url.starts_with("about:") {
            Self::HOUSE
        } else if url.starts_with("http://") {
            Self::GLOBE
        } else {
            Self::GLOBE_SIMPLE
        }
    }
    
    /// Get icon for error type
    pub fn error_icon(error_type: &str) -> &'static str {
        match error_type {
            "NetworkUnreachable" => Self::NETWORK_SLASH,
            "DnsResolutionFailed" => Self::MAGNIFYING_GLASS,
            "ConnectionRefused" => Self::PROHIBIT,
            "ConnectionReset" => Self::ARROWS_CLOCKWISE,
            "TlsHandshakeFailed" => Self::LOCK,
            "TlsVersionMismatch" => Self::LOCK_SIMPLE,
            "CertificateExpired" => Self::CALENDAR_X,
            "HttpUnauthorized" => Self::LOCK_SIMPLE,
            "HttpServerError" => Self::FIRE,
            "HttpRedirectLoop" => Self::ARROWS_CLOCKWISE,
            "ContentTooLarge" => Self::PACKAGE,
            _ => Self::WARNING,
        }
    }
}