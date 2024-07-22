use std::fmt::Display;

struct Response {
    version: Option<String>,
    status: Option<String>,
    explanation: Option<String>,
    body: Option<String>,
}

pub enum Url {
    //  scheme, host, port, path
    Web(String, String, String, String),
    // scheme, path
    File(String, String),
    // scheme, mimetype, data
    Data(String, String, String),
    // Must contain a Url::Web
    ViewSource(Box<Url>),
}

impl Display for Url {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Url::Web(scheme, host, port, path) => {
                write!(f, "{}://{}:{}{}", scheme, host, port, path)
            }
            Url::File(scheme, path) => write!(f, "{}://{}", scheme, path),
            Url::Data(scheme, mimetype, data) => write!(f, "{}://{},{}", scheme, mimetype, data),
            Url::ViewSource(the_source) => write!(f, "view-source:{}", the_source),
        }
    }
}

impl Url {
    pub fn new(url: &str) -> Url {
        let (scheme, url) = url.split_once(':').unwrap();
        match scheme {
            "http" | "https" => {
                let splitter = Splitter { tokens: "//?:/" };
                let mut parts = splitter.split(url);
                // ["", "", host, <port>?, <path>]
                let (_, _, host, port, path) = (
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
                let port = Self::decide_port(scheme, port);
                Url::Web(scheme.to_string(), host.unwrap(), port, path.unwrap())
            }
            "data" => {
                let (mimetype, data) = url
                    .split_once(',')
                    .map(|(first, second)| (first.to_string(), second.to_string()))
                    .unwrap();
                Url::Data(scheme.to_string(), mimetype, data)
            }
            "file" => Url::File(
                scheme.to_string(),
                url.strip_prefix("//").unwrap().to_string(),
            ),
            _ => todo!("the rest"),
        }
    }

    fn decide_port(scheme: &str, port: Option<String>) -> String {
        match port {
            Some(s) if s.is_empty() => Self::default_port(scheme),
            None => Self::default_port(scheme),
            Some(p) => p,
        }
    }

    fn default_port(scheme: &str) -> String {
        match scheme {
            "https" => "443",
            "http" => "80",

            _ => "",
        }
        .to_string()
    }

    // fn request_response(&self) -> Response {

    // }
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
    fn url_exampleorg() {
        let url = Url::new("http://example.org/");
        match url {
            Url::Web(scheme, host, port, path) => {
                assert_eq!(scheme, "http".to_string());
                assert_eq!(host, "example.org".to_string());
                assert_eq!(port, "80".to_string());
                assert_eq!(path, "/");
            }
            _ => unreachable!(),
        };
    }

    #[test]
    fn url_exampleorg_no_slash() {
        let url = Url::new("http://example.org");
        match url {
            Url::Web(scheme, host, port, path) => {
                assert_eq!(scheme, "http".to_string());
                assert_eq!(host, "example.org".to_string());
                assert_eq!(port, "80".to_string());
                assert_eq!(path, "");
            }
            _ => unreachable!(),
        };
    }

    #[test]
    fn url_with_path() {
        let url = Url::new("http://example.org/my/path");
        match url {
            Url::Web(scheme, host, port, path) => {
                assert_eq!(scheme, "http".to_string());
                assert_eq!(host, "example.org".to_string());
                assert_eq!(port, "80".to_string());
                assert_eq!(path, "/my/path");
            }
            _ => unreachable!(),
        };
    }

    #[test]
    fn url_with_host_port() {
        let url = Url::new("http://127.0.0.1:1234/");
        match url {
            Url::Web(scheme, host, port, path) => {
                assert_eq!(scheme, "http".to_string());
                assert_eq!(host, "127.0.0.1".to_string());
                assert_eq!(port, "1234".to_string());
                assert_eq!(path, "/");
            }
            _ => unreachable!(),
        };
    }

    #[test]
    fn url_with_https() {
        let url = Url::new("https://example.org");
        match url {
            Url::Web(scheme, host, port, path) => {
                assert_eq!(scheme, "https".to_string());
                assert_eq!(host, "example.org".to_string());
                assert_eq!(port, "443".to_string());
                assert_eq!(path, "");
            }
            _ => unreachable!(),
        };
    }

    #[test]
    fn url_with_file() {
        let cwd = std::env::current_dir().unwrap();
        let parent_path = cwd.display();
        let url = Url::new(format!("file://{}/data/index.html", parent_path).as_str());

        match url {
            Url::File(scheme, path) => {
                assert_eq!(scheme, "file".to_string());
                assert_eq!(path, format!("{}/data/index.html", parent_path));
            }
            _ => unreachable!(),
        };
    }

    #[test]
    fn data_scheme() {
        let raw_url = "data:text/html,Hello world!";
        let url = Url::new(raw_url);
        match url {
            Url::Data(scheme, mimetype, data) => {
                assert_eq!(scheme, "data".to_string());
                assert_eq!(mimetype, "text/html".to_string());
                assert_eq!(data, "Hello world!".to_string());
            }
            _ => unreachable!(),
        };
    }
}
