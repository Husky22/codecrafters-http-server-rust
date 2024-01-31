// Uncomment this block to pass the first stage
use std::collections::HashMap;
use tokio::{net::{TcpStream, TcpListener}, io::{AsyncWriteExt, AsyncReadExt}};
use anyhow::{Result, Context};
use itertools::Itertools;
mod response;

use response::{HttpResponse, ResponseBody, StatusCode};


async fn handle_stream(mut stream: TcpStream) -> Result<()> {
    let mut input_buf: [u8; 128] = [0; 128];
    stream.read(&mut input_buf).await?;

    let input_string = String::from_utf8(input_buf.to_vec())?;
    let path = extract_path(&input_string);
    let headers = extract_headers(&input_string)?;


    if let Some(p) = path {
        match p.trim_start_matches("/")
            .trim_end_matches("/")
            .split("/")
            .collect_vec()
            .as_slice() 
            {
                [""] => {
                    let response = HttpResponse {
                        status_code: StatusCode::Ok,
                        body: None
                    };
                    stream.write_all(format!("{}", response).as_bytes()).await?;
                    stream.flush().await?;
                },
                ["echo", val @ ..] => {
                    let random_string = val.join("/");
                    let response = HttpResponse {
                        status_code: StatusCode::Ok,
                        body: Some(ResponseBody{
                            content_type: "text/plain".into(),
                            content: random_string
                        })
                    };
                    stream.write_all(format!("{}", response).as_bytes()).await?;
                    stream.flush().await?;

                },
                ["user-agent"] => {
                    let response = HttpResponse {
                        status_code: StatusCode::Ok,
                        body: Some(ResponseBody{
                            content_type: "text/plain".into(),
                            content: headers.get("User-Agent").context("Header User-Agent not found")?.to_owned()
                        })
                    };
                    stream.write_all(format!("{}", response).as_bytes()).await?;
                    stream.flush().await?;

                },
                _ => {
                    let response = HttpResponse {
                        status_code: StatusCode::NotFound,
                        body: None
                    };
                    stream.write_all(format!("{}", response).as_bytes()).await?;
                    stream.flush().await?;
                },

            };
    };
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()>{

    let listener = TcpListener::bind("127.0.0.1:4221").await?;

    loop {
        // The second item contains the IP and port of the new connection.
        let (socket, _) = listener.accept().await.unwrap();
        tokio::spawn(async move { handle_stream(socket).await});
        
    }
}

fn extract_path(input: &str) -> Option<&str> {
    input
        .split("\r\n")
        .take(1)
        .flat_map(|x| x.split(" "))
        .nth(1)
}

fn extract_headers(input: &str) -> Result<HashMap<String, String>> {
    let mut headers = HashMap::new();
    input.split("\r\n\r\n")
        .next().context("No content in input")?
        .split("\r\n")
        .skip(1)
        .filter_map(|x| x.split_once(": "))
        .for_each(|(k,v)| {headers.insert(k.to_owned(), v.to_owned());});

    Ok(headers)
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_extract_path() -> Result<()> {
        let input = "GET /index.html HTTP/1.1\r\nHost: localhost:4221\r\nUser-Agent: curl/7.64.1\r\n\r\n".to_string();

        let path = extract_path(&input);
        assert_eq!(path, Some("/index.html"));
        Ok(())
    }

    #[test]
    fn test_extract_echo_string() {

        let input = "/echo/home".to_string();
        assert_eq!(input.split("/").collect_vec(),vec!["".to_string(), "echo".to_string(), "home".to_string()]);
    }
}
