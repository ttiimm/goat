use std::fmt::Display;

#[derive(Debug)]
pub struct URL {
    scheme: Option<String>,
    host: Option<String>,
    // FIXME: when the type matters
    port: Option<String>,
    path: Option<String>,
}

impl Display for URL {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let scheme = self.scheme.clone().unwrap_or("".to_string());
        let host = self.host.clone().unwrap_or("".to_string());
        let port = self.port.clone().unwrap_or("".to_string());
        let path = self.path.clone().unwrap_or("".to_string());
        write!(f, "{}://{}:{}{}", scheme, host, port, path)
    }
}

impl URL {
    pub fn new(url: &str) -> URL {
        let splitter = Splitter { tokens: "://?:/" };
        let mut parts = splitter.split(url);
        // [scheme, "", "", host, <port>?, <path>]
        let (scheme, _, _, host, port, path) = (
            parts.pop(),
            parts.pop(),
            parts.pop(),
            parts.pop(),
            parts.pop(),
            parts.pop(),
        );
        // always start a path with a slash if not empty
        let path = match path {
            Some(s) if !s.is_empty() => Some(format!("/{}", s)),
            _ => path,
        };
        // if ends with a slash, preserve the slash
        let path = match (url.ends_with('/'), path) {
            (true, Some(s)) if !s.ends_with('/') => Some(format!("{}/", s)),
            (_, path) => path,
        };
        let port = match port {
            Some(s) if s.is_empty() => Self::decide_port(&scheme),
            Some(p) => Some(p),
            None => Self::decide_port(&scheme),
        };
        URL {
            scheme,
            host,
            port,
            path,
        }
    }

    fn decide_port(scheme: &Option<String>) -> Option<String> {
        if *scheme == Some(String::from("https")) {
            Some(String::from("443"))
        } else {
            Some(String::from("80"))
        }
    }
}

struct Splitter<'a> {
    tokens: &'a str,
}

struct Token {
    token: Option<char>,
    la: Option<char>,
}

impl<'a> Splitter<'a> {
    fn split(&self, s: &str) -> Vec<String> {
        let mut parts = Vec::new();
        let mut part = String::from("");
        let mut token_iter = self.tokens.chars();
        let mut next = Self::next(&mut token_iter);

        for c in s.chars() {
            if next.token == Some(c) {
                if next.la.is_some() {
                    next.token = next.la;
                } else {
                    next = Self::next(&mut token_iter);
                }
                parts.push(part);
                part = String::from("");
            } else if next.la == Some(c) {
                next = Self::next(&mut token_iter);
                parts.push(part);
                parts.push(String::from(""));
                part = String::from("");
            } else {
                part.push(c);
            }
        }
        if next.la.is_some() {
            parts.push(part);
            parts.push(String::from(""));
        } else {
            parts.push(part);
        }

        // XXX: not sure this would work with ??
        let expected_len = self.tokens.chars().filter(|t| t != &'?').count() + 1;

        for _ in parts.len()..expected_len {
            parts.push(String::from(""));
        }
        parts.into_iter().rev().collect()
    }

    fn next(token_iter: &mut std::str::Chars) -> Token {
        let next = token_iter.next();
        match next {
            Some('?') => Token {
                token: token_iter.next(),
                la: token_iter.next(),
            },
            _ => Token {
                token: next,
                la: None,
            },
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_url_exampleorg() {
        let url = URL::new("http://example.org/");
        assert_eq!(url.scheme, Some(String::from("http")));
        assert_eq!(url.host, Some(String::from("example.org")));
        assert_eq!(url.path, Some(String::from("/")));
    }

    #[test]
    fn test_url_exampleorg_no_slash() {
        let url = URL::new("http://example.org");
        assert_eq!(url.host, Some(String::from("example.org")));
        assert_eq!(url.path, Some(String::from("")));
    }

    #[test]
    fn test_url_with_path() {
        let url = URL::new("http://example.org/my/path");
        assert_eq!(url.host, Some(String::from("example.org")));
        assert_eq!(url.path, Some(String::from("/my/path")));
    }

    #[test]
    fn test_url_with_host_port() {
        let url = URL::new("http://127.0.0.1:1234/");
        assert_eq!(url.host, Some(String::from("127.0.0.1")));
        assert_eq!(url.port, Some(String::from("1234")));
    }

    #[test]
    fn test_url_with_https() {
        let url = URL::new("https://example.org");
        assert_eq!(url.scheme, Some(String::from("https")));
        assert_eq!(url.port, Some(String::from("443")));
    }

    #[test]
    fn test_url_with_file() {
        let cwd = std::env::current_dir().unwrap();
        let parent_path = cwd.display();
        let url = URL::new(format!("file://{}/data/index.html", parent_path).as_str());
        assert_eq!(url.scheme, Some(String::from("file")));
        assert_eq!(url.path, Some(format!("{}/data/index.html", parent_path)));
    }
}
