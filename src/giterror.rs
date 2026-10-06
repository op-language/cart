//! Fix hints for git clone and fetch failures.
//!
//! The helper maps libgit2 error classes and codes to a short hint that
//! tells the user how to fix or work around the problem. Call sites append
//! the hint to the E510 error text.

use std::path::Path;

/// Returns a fix hint for a failed git clone or fetch. Returns `None`
/// when the error has no known hint.
pub fn clone_error_hint(err: &git2::Error, url: &str, dest: &Path) -> Option<String> {
    // libgit2 reports a missing TLS backend with this fixed message and
    // no dedicated error code, so the dispatch has to match the message.
    if err.class() == git2::ErrorClass::Ssl && err.message() == "there is no TLS stream available" {
        return Some(format!(
            "This cart binary was built without HTTPS support. Rebuild and \
             reinstall cart from source. System git workaround for this URL: \
             git clone {url} {}",
            dest.display()
        ));
    }
    if err.code() == git2::ErrorCode::Certificate {
        return Some(
            "The HTTPS certificate check failed. Check the system CA \
             certificates and the proxy settings."
                .to_string(),
        );
    }
    if err.code() == git2::ErrorCode::Auth {
        return Some(
            "The git server rejected the credentials. Check the stored \
             credentials for this URL."
                .to_string(),
        );
    }
    if err.class() == git2::ErrorClass::Net && err.message().contains("failed to resolve address") {
        return Some(
            "cart could not resolve the server address. Check the network \
             connection and the server address."
                .to_string(),
        );
    }
    None
}

/// Formats the fix hint as an extra `hint:` line for the error text.
/// Returns an empty string when the error has no hint.
pub fn hint_lines(err: &git2::Error, url: &str, dest: &Path) -> String {
    match clone_error_hint(err, url, dest) {
        Some(hint) => format!("\nhint: {hint}"),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::{clone_error_hint, hint_lines};
    use std::path::PathBuf;

    fn dest() -> PathBuf {
        PathBuf::from("/home/u/.cart/std")
    }

    fn tls_missing_error() -> git2::Error {
        git2::Error::new(
            git2::ErrorCode::GenericError,
            git2::ErrorClass::Ssl,
            "there is no TLS stream available",
        )
    }

    #[test]
    fn tls_missing_hint_tells_rebuild_and_manual_clone() {
        let hint = clone_error_hint(
            &tls_missing_error(),
            "https://github.com/op-language/std",
            &dest(),
        )
        .expect("TLS-missing errors need a hint");
        assert!(hint.contains("without HTTPS support"));
        assert!(hint.contains("Rebuild and reinstall cart from source."));
        assert!(hint.contains("git clone https://github.com/op-language/std /home/u/.cart/std"));
    }

    #[test]
    fn hint_lines_formats_prefix() {
        let text = hint_lines(
            &tls_missing_error(),
            "https://github.com/op-language/std",
            &dest(),
        );
        assert!(text.starts_with("\nhint: "));
    }

    #[test]
    fn certificate_hint_names_cas_and_proxy() {
        let err = git2::Error::new(
            git2::ErrorCode::Certificate,
            git2::ErrorClass::Ssl,
            "certificate check failed",
        );
        let hint = clone_error_hint(&err, "https://example.com/x", &dest())
            .expect("certificate errors need a hint");
        assert!(hint.contains("CA certificates"));
        assert!(hint.contains("proxy"));
    }

    #[test]
    fn auth_hint_mentions_credentials() {
        let err = git2::Error::new(
            git2::ErrorCode::Auth,
            git2::ErrorClass::Net,
            "authentication failed",
        );
        let hint = clone_error_hint(&err, "https://example.com/x", &dest())
            .expect("auth errors need a hint");
        assert!(hint.contains("credentials"));
    }

    #[test]
    fn dns_failure_hint_mentions_network() {
        let err = git2::Error::new(
            git2::ErrorCode::GenericError,
            git2::ErrorClass::Net,
            "failed to resolve address for example.com: Name or service not known",
        );
        let hint = clone_error_hint(&err, "https://example.com/x", &dest())
            .expect("DNS failures need a hint");
        assert!(hint.contains("network"));
    }

    #[test]
    fn net_error_without_dns_failure_gets_no_hint() {
        let err = git2::Error::new(
            git2::ErrorCode::GenericError,
            git2::ErrorClass::Net,
            "connection reset by peer",
        );
        let hint = clone_error_hint(&err, "https://example.com/x", &dest());
        assert!(hint.is_none());
    }

    #[test]
    fn unknown_error_gets_no_hint() {
        let err = git2::Error::new(
            git2::ErrorCode::GenericError,
            git2::ErrorClass::Repository,
            "repository is empty",
        );
        let hint = clone_error_hint(&err, "https://example.com/x", &dest());
        assert!(hint.is_none());
        assert_eq!(hint_lines(&err, "https://example.com/x", &dest()), "");
    }
}
