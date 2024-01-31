use std::{collections::HashMap, path::Path, sync::Arc};
use tokio::{net::{TcpStream, TcpListener}, io::{AsyncWriteExt, AsyncReadExt}, fs};
use anyhow::{Result, Context};
use itertools::Itertools;
use clap::Parser;


mod response;

use response::{HttpResponse, ResponseBody, StatusCode};


const NOT_FOUND_RESPONSE: HttpResponse = HttpResponse {
    status_code: StatusCode::NotFound,
    body: None
};

async fn send(mut stream: TcpStream, response: HttpResponse) -> Result<()> {
    stream.write_all(format!("{}", response).as_bytes()).await?;
    stream.flush().await?;
    Ok(())
}


async fn handle_stream(mut stream: TcpStream, directory: Arc<Option<Box<Path>>>) -> Result<()> {
    let mut input_buf: [u8; 1024] = [0; 1024];
    stream.read(&mut input_buf).await?;

    let input_string = String::from_utf8(input_buf.to_vec())?
        .trim_matches(char::from(0))
        .to_owned();
    
    let path = extract_path(&input_string);
    let request_type = extract_request_type(&input_string).context("No Request Type")?;
    let headers = extract_headers(&input_string)?;


    if let Some(p) = path {
        match (request_type, p.trim_start_matches("/")
            .trim_end_matches("/")
            .split("/")
            .collect_vec()
            .as_slice())
            {
                ("GET", [""]) => {
                    let response = HttpResponse {
                        status_code: StatusCode::Ok,
                        body: None
                    };
                    send(stream, response).await?;
                },
                ("GET", ["echo", val @ ..]) => {
                    let random_string = val.join("/");
                    let response = HttpResponse {
                        status_code: StatusCode::Ok,
                        body: Some(ResponseBody{
                            content_type: "text/plain".into(),
                            content: random_string
                        })
                    };
                    send(stream, response).await?;
                },
                ("GET", ["user-agent"]) => {
                    let response = HttpResponse {
                        status_code: StatusCode::Ok,
                        body: Some(ResponseBody{
                            content_type: "text/plain".into(),
                            content: headers.get("User-Agent").context("Header User-Agent not found")?.to_owned()
                        })
                    };
                    send(stream, response).await?;

                },
                ("GET", ["files", val @ ..]) => {
                    let file_path = directory.as_ref().as_ref().map(|s| s.join(val.join("/")));

                    match file_path {
                        Some(dir) => {
                            if dir.exists() {
                                let content = fs::read_to_string(dir).await?;
                                let response = HttpResponse {
                                    status_code: StatusCode::Ok,
                                    body: Some(ResponseBody{
                                        content_type: "application/octet-stream".into(),
                                        content
                                    })
                                };
                                send(stream, response).await?;

                            } else {
                                send(stream, NOT_FOUND_RESPONSE).await?;
                            }
                        },
                        None => send(stream, NOT_FOUND_RESPONSE).await?,
                    }
                },
                ("POST", ["files", val @ ..]) => {
                    let file_path = directory.as_ref().as_ref().map(|s| s.join(val.join("/")));
                    let contents = extract_body(&input_string).context("no body found")?;


                    match file_path {
                        Some(dir) => {
                            if !dir.exists() {
                                fs::write(dir, contents).await?;
                                let response = HttpResponse {
                                    status_code: StatusCode::Created,
                                    body: None
                                };
                                send(stream, response).await?;

                            } else {
                                send(stream, NOT_FOUND_RESPONSE).await?;
                            }
                        },
                        None => send(stream, NOT_FOUND_RESPONSE).await?,
                    }



                },
                _ => {
                    send(stream, NOT_FOUND_RESPONSE).await?;
                },

            };
    };
    Ok(())
}


#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long)]
    directory: Option<Box<Path>>
}

#[tokio::main]
async fn main() -> Result<()>{
    let args = Args::parse();

    let dir = Arc::new(args.directory);

    let listener = TcpListener::bind("127.0.0.1:4221").await?;

    loop {
        let dir = dir.clone();
        // The second item contains the IP and port of the new connection.
        let (socket, _) = listener.accept().await.unwrap();
        tokio::spawn(async move { handle_stream(socket, dir).await});
        
    }
}

fn extract_path(input: &str) -> Option<&str> {
    input
        .split("\r\n")
        .take(1)
        .flat_map(|x| x.split(" "))
        .nth(1)
}

fn extract_body(input: &str) -> Option<&str> {
    input.split("\r\n\r\n")
        .nth(1)
}

fn extract_request_type(input: &str) -> Option<&str> {
    input
        .split("\r\n")
        .take(1)
        .flat_map(|x| x.split(" "))
        .nth(0)
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
