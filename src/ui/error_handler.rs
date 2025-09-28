use std::fmt;
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct BrowserError {
    pub error_type: ErrorType,
    pub message: String,
    pub url: Option<String>,
    pub suggestions: Vec<String>,
    pub retry_possible: bool,
    pub technical_details: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorType {
    // Network errors
    NetworkTimeout,
    NetworkUnreachable,
    DnsResolutionFailed,
    ConnectionRefused,
    ConnectionReset,
    
    // TLS/Security errors
    TlsHandshakeFailed,
    CertificateInvalid,
    CertificateExpired,
    TlsVersionMismatch,
    
    // HTTP errors
    HttpBadRequest,        // 400
    HttpUnauthorized,      // 401
    HttpForbidden,         // 403
    HttpNotFound,          // 404
    HttpServerError,       // 5xx
    HttpRedirectLoop,
    
    // Content errors
    ContentTooLarge,
    ContentMalformed,
    ContentUnsupported,
    
    // Browser errors
    InvalidUrl,
    UnsupportedProtocol,
    ResourceExhausted,
    InternalError,
}

impl fmt::Display for ErrorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            ErrorType::NetworkTimeout => "Network Timeout",
            ErrorType::NetworkUnreachable => "Network Unreachable",
            ErrorType::DnsResolutionFailed => "DNS Resolution Failed",
            ErrorType::ConnectionRefused => "Connection Refused",
            ErrorType::ConnectionReset => "Connection Reset",
            ErrorType::TlsHandshakeFailed => "TLS Handshake Failed",
            ErrorType::CertificateInvalid => "Certificate Invalid",
            ErrorType::CertificateExpired => "Certificate Expired",
            ErrorType::TlsVersionMismatch => "TLS Version Mismatch",
            ErrorType::HttpBadRequest => "Bad Request (400)",
            ErrorType::HttpUnauthorized => "Unauthorized (401)",
            ErrorType::HttpForbidden => "Forbidden (403)",
            ErrorType::HttpNotFound => "Not Found (404)",
            ErrorType::HttpServerError => "Server Error (5xx)",
            ErrorType::HttpRedirectLoop => "Redirect Loop",
            ErrorType::ContentTooLarge => "Content Too Large",
            ErrorType::ContentMalformed => "Content Malformed",
            ErrorType::ContentUnsupported => "Content Unsupported",
            ErrorType::InvalidUrl => "Invalid URL",
            ErrorType::UnsupportedProtocol => "Unsupported Protocol",
            ErrorType::ResourceExhausted => "Resource Exhausted",
            ErrorType::InternalError => "Internal Error",
        };
        write!(f, "{}", msg)
    }
}

impl BrowserError {
    pub fn new(error_type: ErrorType, message: String) -> Self {
        Self {
            error_type,
            message,
            url: None,
            suggestions: Vec::new(),
            retry_possible: false,
            technical_details: None,
        }
    }

    pub fn with_url(mut self, url: String) -> Self {
        self.url = Some(url);
        self
    }

    pub fn with_suggestions(mut self, suggestions: Vec<String>) -> Self {
        self.suggestions = suggestions;
        self
    }

    pub fn with_retry(mut self) -> Self {
        self.retry_possible = true;
        self
    }

    pub fn with_technical_details(mut self, details: String) -> Self {
        self.technical_details = Some(details);
        self
    }

    pub fn from_network_error(err: &str, url: &str) -> Self {
        let (error_type, user_message, suggestions) = if err.contains("timeout") {
            (
                ErrorType::NetworkTimeout,
                "The connection timed out".to_string(),
                vec![
                    "Check your internet connection".to_string(),
                    "Try again in a few moments".to_string(),
                    "The server might be temporarily overloaded".to_string(),
                ]
            )
        } else if err.contains("dns") || err.contains("resolve") || err.contains("name") {
            (
                ErrorType::DnsResolutionFailed,
                "Could not find the website".to_string(),
                vec![
                    "Check if the URL is spelled correctly".to_string(),
                    "Try using a different DNS server (8.8.8.8)".to_string(),
                    "The website might be temporarily unavailable".to_string(),
                ]
            )
        } else if err.contains("refused") || err.contains("connect") {
            (
                ErrorType::ConnectionRefused,
                "The server refused the connection".to_string(),
                vec![
                    "The website might be down for maintenance".to_string(),
                    "Try again later".to_string(),
                    "Check if the URL is correct".to_string(),
                ]
            )
        } else if err.contains("reset") {
            (
                ErrorType::ConnectionReset,
                "The connection was reset by the server".to_string(),
                vec![
                    "Try refreshing the page".to_string(),
                    "The server might be overloaded".to_string(),
                    "Check your firewall settings".to_string(),
                ]
            )
        } else {
            (
                ErrorType::NetworkUnreachable,
                "Network error occurred".to_string(),
                vec![
                    "Check your internet connection".to_string(),
                    "Try again in a few moments".to_string(),
                ]
            )
        };

        Self::new(error_type, user_message)
            .with_url(url.to_string())
            .with_suggestions(suggestions)
            .with_retry()
            .with_technical_details(err.to_string())
    }

    pub fn from_tls_error(err: &str, url: &str) -> Self {
        let (error_type, user_message, suggestions) = if err.contains("certificate") {
            if err.contains("expired") {
                (
                    ErrorType::CertificateExpired,
                    "The website's security certificate has expired".to_string(),
                    vec![
                        "The website owner needs to renew their certificate".to_string(),
                        "Your system date/time might be incorrect".to_string(),
                        "Try accessing the site later".to_string(),
                    ]
                )
            } else {
                (
                    ErrorType::CertificateInvalid,
                    "The website's security certificate is invalid".to_string(),
                    vec![
                        "This could be a security risk".to_string(),
                        "Make sure you're visiting the correct website".to_string(),
                        "Contact the website administrator".to_string(),
                    ]
                )
            }
        } else if err.contains("protocol") || err.contains("version") {
            (
                ErrorType::TlsVersionMismatch,
                "TLS protocol version not supported".to_string(),
                vec![
                    "The website uses an outdated security protocol".to_string(),
                    "Try using a different browser".to_string(),
                    "Contact the website administrator".to_string(),
                ]
            )
        } else if err.contains("close_notify") {
            (
                ErrorType::TlsHandshakeFailed,
                "Secure connection was closed unexpectedly".to_string(),
                vec![
                    "Try refreshing the page".to_string(),
                    "The server might be temporarily unavailable".to_string(),
                    "Check your internet connection".to_string(),
                ]
            )
        } else {
            (
                ErrorType::TlsHandshakeFailed,
                "Could not establish a secure connection".to_string(),
                vec![
                    "Try refreshing the page".to_string(),
                    "Check your system date and time".to_string(),
                    "Contact the website administrator".to_string(),
                ]
            )
        };

        Self::new(error_type, user_message)
            .with_url(url.to_string())
            .with_suggestions(suggestions)
            .with_retry()
            .with_technical_details(err.to_string())
    }

    pub fn from_http_status(status_code: u16, url: &str) -> Self {
        let (error_type, user_message, suggestions) = match status_code {
            400 => (
                ErrorType::HttpBadRequest,
                "Bad request - the server couldn't understand the request".to_string(),
                vec![
                    "Check if the URL is correct".to_string(),
                    "Try refreshing the page".to_string(),
                ]
            ),
            401 => (
                ErrorType::HttpUnauthorized,
                "Authentication required".to_string(),
                vec![
                    "You need to log in to access this page".to_string(),
                    "Check your credentials".to_string(),
                ]
            ),
            403 => (
                ErrorType::HttpForbidden,
                "Access forbidden".to_string(),
                vec![
                    "You don't have permission to view this page".to_string(),
                    "Try logging in with a different account".to_string(),
                    "Contact the website administrator".to_string(),
                ]
            ),
            404 => (
                ErrorType::HttpNotFound,
                "Page not found".to_string(),
                vec![
                    "Check if the URL is spelled correctly".to_string(),
                    "The page might have been moved or deleted".to_string(),
                    "Try searching for the content on the website".to_string(),
                ]
            ),
            500..=599 => (
                ErrorType::HttpServerError,
                format!("Server error ({})", status_code),
                vec![
                    "The website is experiencing technical difficulties".to_string(),
                    "Try again in a few minutes".to_string(),
                    "Contact the website administrator if the problem persists".to_string(),
                ]
            ),
            _ => (
                ErrorType::InternalError,
                format!("HTTP error {}", status_code),
                vec![
                    "An unexpected error occurred".to_string(),
                    "Try refreshing the page".to_string(),
                ]
            )
        };

        let retry = matches!(status_code, 500..=599 | 408 | 429);

        let mut error = Self::new(error_type, user_message)
            .with_url(url.to_string())
            .with_suggestions(suggestions)
            .with_technical_details(format!("HTTP Status Code: {}", status_code));

        if retry {
            error = error.with_retry();
        }

        error
    }

    pub fn from_anyhow(err: &anyhow::Error, url: Option<&str>) -> Self {
        let err_str = err.to_string();
        
        // Try to categorize the error based on the message
        if err_str.contains("TLS") || err_str.contains("SSL") || err_str.contains("certificate") {
            Self::from_tls_error(&err_str, url.unwrap_or("unknown"))
        } else if err_str.contains("timeout") || err_str.contains("dns") || err_str.contains("connect") {
            Self::from_network_error(&err_str, url.unwrap_or("unknown"))
        } else if err_str.starts_with("REDIRECT:") {
            Self::new(
                ErrorType::HttpRedirectLoop,
                "Too many redirects".to_string()
            )
            .with_url(url.unwrap_or("unknown").to_string())
            .with_suggestions(vec![
                "The website has a redirect loop".to_string(),
                "Clear your browser cookies and try again".to_string(),
                "Contact the website administrator".to_string(),
            ])
        } else {
            Self::new(
                ErrorType::InternalError,
                "An unexpected error occurred".to_string()
            )
            .with_suggestions(vec![
                "Try refreshing the page".to_string(),
                "Check your internet connection".to_string(),
            ])
            .with_technical_details(err_str)
        }
    }

    pub fn icon(&self) -> &'static str {
        match self.error_type {
            ErrorType::NetworkTimeout => "⏱️",
            ErrorType::NetworkUnreachable => "🌐",
            ErrorType::DnsResolutionFailed => "🔍",
            ErrorType::ConnectionRefused => "🚫",
            ErrorType::ConnectionReset => "🔄",
            ErrorType::TlsHandshakeFailed => "🔒",
            ErrorType::CertificateInvalid => "⚠️",
            ErrorType::CertificateExpired => "📅",
            ErrorType::TlsVersionMismatch => "🔐",
            ErrorType::HttpBadRequest => "❌",
            ErrorType::HttpUnauthorized => "🔐",
            ErrorType::HttpForbidden => "⛔",
            ErrorType::HttpNotFound => "❓",
            ErrorType::HttpServerError => "🔥",
            ErrorType::HttpRedirectLoop => "🔄",
            ErrorType::ContentTooLarge => "📦",
            ErrorType::ContentMalformed => "⚠️",
            ErrorType::ContentUnsupported => "❌",
            ErrorType::InvalidUrl => "🔗",
            ErrorType::UnsupportedProtocol => "⚠️",
            ErrorType::ResourceExhausted => "💾",
            ErrorType::InternalError => "🐛",
        }
    }

    pub fn color(&self) -> egui::Color32 {
        use crate::ui::theme::NeonTheme;
        match self.error_type {
            ErrorType::NetworkTimeout | ErrorType::NetworkUnreachable | ErrorType::ConnectionReset => {
                NeonTheme::WARNING_COLOR
            },
            ErrorType::CertificateInvalid | ErrorType::CertificateExpired | ErrorType::TlsHandshakeFailed => {
                NeonTheme::ERROR_COLOR
            },
            ErrorType::HttpNotFound | ErrorType::HttpForbidden => {
                NeonTheme::WARNING_COLOR
            },
            ErrorType::HttpServerError => {
                NeonTheme::ERROR_COLOR
            },
            _ => NeonTheme::ERROR_COLOR,
        }
    }
}

impl fmt::Display for BrowserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for BrowserError {}

// Error recovery and diagnostic utilities
pub struct ErrorRecovery;

impl ErrorRecovery {
    pub async fn diagnose_connectivity() -> Vec<String> {
        let mut diagnostics = Vec::new();
        
        // Check basic connectivity
        diagnostics.push("🔍 Checking network connectivity...".to_string());
        
        // Try to resolve some common domains
        let test_domains = vec!["google.com", "cloudflare.com", "github.com"];
        for domain in test_domains {
            match tokio::net::lookup_host((domain, 80)).await {
                Ok(_) => diagnostics.push(format!("✅ {} is reachable", domain)),
                Err(_) => diagnostics.push(format!("❌ {} is not reachable", domain)),
            }
        }
        
        diagnostics
    }
    
    pub fn suggest_fixes(error: &BrowserError) -> Vec<String> {
        let mut fixes = error.suggestions.clone();
        
        // Add general fixes based on error type
        match error.error_type {
            ErrorType::DnsResolutionFailed => {
                fixes.extend(vec![
                    "Try using Google DNS (8.8.8.8, 8.8.4.4)".to_string(),
                    "Flush your DNS cache".to_string(),
                    "Check your router settings".to_string(),
                ]);
            },
            ErrorType::NetworkTimeout => {
                fixes.extend(vec![
                    "Disable VPN temporarily".to_string(),
                    "Check firewall settings".to_string(),
                    "Try from a different network".to_string(),
                ]);
            },
            ErrorType::TlsHandshakeFailed => {
                fixes.extend(vec![
                    "Update your system date/time".to_string(),
                    "Clear browser cache and cookies".to_string(),
                    "Check antivirus/firewall settings".to_string(),
                ]);
            },
            _ => {}
        }
        
        fixes
    }
}