use std::fmt::Display;

pub enum StatusCode {
    Ok,
    NotFound
}

pub struct HttpResponse {
    pub status_code: StatusCode,
    pub body: Option<ResponseBody>
}

pub struct ResponseBody {
    pub content_type: String,
    pub content: String,
}


impl Display for ResponseBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Content-Type: {}\r\nContent-Length: {}\r\n\r\n{}",
               self.content_type,
               self.content.len(),
               self.content)
    }
}


impl Display for HttpResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.status_code {
            StatusCode::Ok => {
                let mut lines = format!("HTTP/1.1 200 OK");

                if let Some(body) = &self.body {
                    lines = format!("{}\r\n{}", lines, body);
                }
                write!(f, "{}\r\n\r\n", lines)
            },
            StatusCode::NotFound => {
                write!(f, "HTTP/1.1 404 NOT FOUND\r\n\r\n")
            },
        }
    }
}
