//! Hypertext Transfer Protocol (HTTP/1.1 Client).

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpRequest {
    pub method: &'static str,
    pub path: String,
    pub host: String,
}

impl HttpRequest {
    pub fn new_get(host: &str, path: &str) -> Self {
        Self {
            method: "GET",
            path: path.to_string(),
            host: host.to_string(),
        }
    }

    pub fn to_http_wire(&self) -> String {
        format!(
            "{} {} HTTP/1.1\r\nHost: {}\r\nUser-Agent: Micro-OS-Rust/0.1\r\nConnection: close\r\n\r\n",
            self.method, self.path, self.host
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpResponse {
    pub status_code: u16,
    pub status_text: String,
    pub body: String,
}

impl HttpResponse {
    pub fn parse(raw: &str) -> Result<Self, &'static str> {
        let mut lines = raw.lines();
        let status_line = lines.next().ok_or("Empty HTTP response")?;
        let parts: Vec<&str> = status_line.splitn(3, ' ').collect();
        if parts.len() < 2 {
            return Err("Malformed HTTP status line");
        }
        let code = parts[1].parse::<u16>().map_err(|_| "Invalid status code")?;
        let text = if parts.len() >= 3 { parts[2] } else { "" };

        let body = if let Some(pos) = raw.find("\r\n\r\n") {
            &raw[pos + 4..]
        } else {
            ""
        };

        Ok(Self {
            status_code: code,
            status_text: text.to_string(),
            body: body.to_string(),
        })
    }
}
