use url::Url;

use crate::internal::routehasher::domain::errors::URLDBError;

pub const MAX_URL_LEN: usize = 2048;

/// Checks a URL before it is shortened and returns the normalized form to store.
///
/// The redirect sends visitors straight to whatever is stored, so only plain `http` / `https`
/// links with a host are allowed. That rules out `javascript:`, `data:`, `file:` and similar.
/// Credentials in the URL (`https://bank.com@evil.example`) are rejected too, since they are
/// a common way to disguise where a link really goes.
pub fn check_url(raw: &str) -> Result<String, URLDBError> {
    let raw = raw.trim();

    if raw.is_empty() {
        return Err(URLDBError::InvalidUrl("url is required".to_owned()));
    }
    if raw.len() > MAX_URL_LEN {
        return Err(URLDBError::InvalidUrl(format!("url is longer than {MAX_URL_LEN} characters")));
    }

    let url = Url::parse(raw)
        .map_err(|_| URLDBError::InvalidUrl("url is not a valid absolute URL".to_owned()))?;

    if !matches!(url.scheme(), "http" | "https") {
        return Err(URLDBError::InvalidUrl("only http and https urls can be shortened".to_owned()));
    }
    if url.host_str().is_none_or(str::is_empty) {
        return Err(URLDBError::InvalidUrl("url must include a host".to_owned()));
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err(URLDBError::InvalidUrl(
            "urls with embedded credentials are not allowed".to_owned(),
        ));
    }

    Ok(url.into())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn accepts_http_and_https() {
        assert_eq!(check_url("https://example.com/a?b=1").unwrap(), "https://example.com/a?b=1");
        assert_eq!(check_url("  http://example.com  ").unwrap(), "http://example.com/");
    }

    #[test]
    fn rejects_dangerous_schemes() {
        for bad in ["javascript:alert(1)", "data:text/html,hi", "file:///etc/passwd", "ftp://x.com"]
        {
            assert!(matches!(check_url(bad), Err(URLDBError::InvalidUrl(_))), "{bad}");
        }
    }

    #[test]
    fn rejects_relative_empty_and_credentials() {
        for bad in ["", "   ", "example.com", "/path", "https://user:pw@example.com"] {
            assert!(matches!(check_url(bad), Err(URLDBError::InvalidUrl(_))), "{bad:?}");
        }
    }

    #[test]
    fn rejects_too_long() {
        let long = format!("https://example.com/{}", "a".repeat(MAX_URL_LEN));
        assert!(matches!(check_url(&long), Err(URLDBError::InvalidUrl(_))));
    }
}
