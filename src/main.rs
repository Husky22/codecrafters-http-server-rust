// Uncomment this block to pass the first stage
use std::{net::TcpListener, io::{Write, Read}};
use anyhow::Result;
use itertools::Itertools;

fn main() -> Result<()>{
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    //
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("accepted new connection");

                let mut input_buf: [u8; 128] = [0; 128];
                stream.read(&mut input_buf)?;

                let input_string = String::from_utf8(input_buf.to_vec())?;
                let path = extract_path(&input_string);
                match path {
                    Some("/") => {
                        stream.write(b"HTTP/1.1 200 OK\r\n\r\n")?;
                        stream.flush()?;
                    },
                    Some(a) => {
                        dbg!(a);
                        let mut args = a.split("/").skip(1);

                        match args.next() {
                            Some("echo") => {
                                let random_string = args.join("/");
                                write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}\r\n", random_string.len(), random_string)?;
                                dbg!(random_string.clone());
                                stream.flush()?;

                            },
                            _ => {
                                stream.write(b"HTTP/1.1 404 NOT FOUND\r\n\r\n")?;
                                stream.flush()?;
                            },
                        }
                    },
                    None => todo!(),
                }


            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }

    Ok(())
}

fn extract_path(input: &str) -> Option<&str> {
    input
        .split("\r\n")
        .take(1)
        .flat_map(|x| x.split(" "))
        .nth(1)
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
        assert_eq!(input.split("/").collect_vec(),vec!["echo".to_string(), "home".to_string()]);
    }
}
