use std::io::prelude::*;
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::{collections::HashMap, fmt::Display};

struct Response {
    version: String,
    status: String,
    explanation: String,
    headers: HashMap<String, String>,
    body: Option<String>,
}

pub enum Url {
    Web {
        scheme: String,
        host: String,
        port: u16,
        path: String,
    },
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
            Url::Web {
                scheme,
                host,
                port,
                path,
            } => {
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
        let (scheme, mut url) = url.split_once(':').unwrap();
        match scheme {
            "http" | "https" => {
                url = url.strip_prefix("//").unwrap();
                let (host_port, path) = match url.split_once('/') {
                    Some(result) => result,
                    None => (url, ""),
                };
                let (host, port) = match host_port.split_once(':') {
                    Some(result) => result,
                    None => (host_port, Self::default_port(scheme)),
                };

                // always start a path with a slash if not empty
                let path = match path {
                    s if !s.is_empty() => format!("/{}", s),
                    _ => path.to_string(),
                };
                // if ends with a slash, preserve the slash
                let path = match (url.ends_with('/'), path) {
                    (true, s) if !s.ends_with('/') => format!("{}/", s),
                    (_, path) => path,
                };
                Url::Web {
                    scheme: scheme.to_string(),
                    host: host.to_string(),
                    port: port.parse().expect("todo"),
                    path: path.to_string(),
                }
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
            "view-source" => Url::ViewSource(Box::new(Url::new(url))),
            _ => todo!("the rest"),
        }
    }

    fn default_port(scheme: &str) -> &str {
        match scheme {
            "https" => "443",
            "http" => "80",

            _ => "",
        }
    }
}

// impl for Url::Web
impl Url {
    fn build_socket_addr(&self) -> SocketAddr {
        match self {
            Url::Web {
                scheme: _,
                host,
                port,
                path: _,
            } => {
                let addrs = format!("{}:{}", host, port).to_socket_addrs().unwrap();
                addrs.into_iter().next().expect("todo")
            }
            _ => unreachable!(),
        }
    }
}

enum ResponseError {
    Socket(std::io::Error),
}

impl From<std::io::Error> for ResponseError {
    fn from(value: std::io::Error) -> Self {
        ResponseError::Socket(value)
    }
}

impl std::error::Error for ResponseError {}

impl std::fmt::Debug for ResponseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Socket(err) => f.debug_tuple("Socket").field(err).finish(),
        }
    }
}

impl std::fmt::Display for ResponseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO: something else
        write!(f, "{:?}", self)
    }
}

impl Url {
    fn request_response(&self) -> Result<Response, ResponseError> {
        match self {
            Url::Web {
                scheme: _,
                host,
                port: _,
                path,
            } => {
                let addr = self.build_socket_addr();
                let mut stream = TcpStream::connect(addr)?;
                stream.write_all(format!("GET {path} HTTP/1.0\r\n").as_bytes())?;
                stream.write_all(format!("HOST {host}\r\n").as_bytes())?;
                stream.write_all("User-Agent: Goat\r\n".as_bytes())?;
                stream.write_all("\r\n".as_bytes())?;
                // stream.read(&mut [0; 128])?;
                Ok(Response {
                    version: "".to_string(),
                    status: "".to_string(),
                    explanation: "".to_string(),
                    headers: HashMap::new(),
                    body: Some("".to_string()),
                })
            }
            Url::File(_, _) => todo!(),
            Url::Data(_, _, _) => todo!(),
            Url::ViewSource(_) => todo!(),
        }
    }
}

#[cfg(test)]
mod tests {

    use httpmock::{Method::GET, MockServer};

    use super::*;

    #[test]
    fn url_exampleorg() {
        let url = Url::new("http://example.org/");
        match url {
            Url::Web {
                scheme,
                host,
                port,
                path,
            } => {
                assert_eq!(scheme, "http".to_string());
                assert_eq!(host, "example.org".to_string());
                assert_eq!(port, 80);
                assert_eq!(path, "/");
            }
            _ => unreachable!(),
        };
    }

    #[test]
    fn url_exampleorg_no_slash() {
        let url = Url::new("http://example.org");
        match url {
            Url::Web {
                scheme,
                host,
                port,
                path,
            } => {
                assert_eq!(scheme, "http".to_string());
                assert_eq!(host, "example.org".to_string());
                assert_eq!(port, 80);
                assert_eq!(path, "");
            }
            _ => unreachable!(),
        };
    }

    #[test]
    fn url_with_path() {
        let url = Url::new("http://example.org/my/path");
        match url {
            Url::Web {
                scheme,
                host,
                port,
                path,
            } => {
                assert_eq!(scheme, "http".to_string());
                assert_eq!(host, "example.org".to_string());
                assert_eq!(port, 80);
                assert_eq!(path, "/my/path");
            }
            _ => unreachable!(),
        };
    }

    #[test]
    fn url_with_host_port() {
        let url = Url::new("http://127.0.0.1:1234/");
        match url {
            Url::Web {
                scheme,
                host,
                port,
                path,
            } => {
                assert_eq!(scheme, "http".to_string());
                assert_eq!(host, "127.0.0.1".to_string());
                assert_eq!(port, 1234);
                assert_eq!(path, "/");
            }
            _ => unreachable!(),
        };
    }

    #[test]
    fn url_with_host_port_path() {
        let url = Url::new("http://127.0.0.1:1234/my/path/hello");
        match url {
            Url::Web {
                scheme,
                host,
                port,
                path,
            } => {
                assert_eq!(scheme, "http".to_string());
                assert_eq!(host, "127.0.0.1".to_string());
                assert_eq!(port, 1234);
                assert_eq!(path, "/my/path/hello");
            }
            _ => unreachable!(),
        };
    }

    #[test]
    fn url_with_https() {
        let url = Url::new("https://example.org");
        match url {
            Url::Web {
                scheme,
                host,
                port,
                path,
            } => {
                assert_eq!(scheme, "https".to_string());
                assert_eq!(host, "example.org".to_string());
                assert_eq!(port, 443);
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

    #[test]
    fn view_source() {
        let raw_url = "view-source:http://localhost:8888/data/index.html";
        let url = Url::new(raw_url);
        let the_source = match url {
            Url::ViewSource(the_source) => the_source,
            _ => unreachable!(),
        };

        match *the_source {
            Url::Web {
                scheme,
                host,
                port,
                path,
            } => {
                assert_eq!(scheme, "http".to_string());
                assert_eq!(host, "localhost".to_string());
                assert_eq!(port, 8888);
                assert_eq!(path, "/data/index.html");
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn request_response() {
        let server = MockServer::start();

        let mock = server.mock(|when, then| {
            when.method(GET).path("/data/index.html");
            then.status(200).body("<html>hi</html>");
        });

        let url = Url::new(server.url("/data/index.html").as_str());
        let response = url.request_response().unwrap();
        assert_eq!(response.version, "HTTP/1.0");
        assert_eq!(response.status, "200");
        assert_eq!(response.explanation, "OK\r\n");
        assert_eq!(response.headers["content-type"], "text/html");
        assert_eq!(response.body, Some("<html>hi</html>".to_string()));
        mock.assert_hits(1);
        // assert_eq!(url.num_sockets(), 1);
    }
}
