use std::{error::Error, fs, io::{BufRead, BufReader, Write}, net::{TcpListener, TcpStream}, thread, time::Duration};
use yarws::ThreadPool;
const LOCALHOST: &str = "127.0.0.1";
const PORT: &str = "7878";
const OK_RESPONSE: &str = "HTTP/1.1 200 OK\r\n\r\n";
const NOT_FOUND_RESPONSE: &str = "HTTP/1.1 404 NOT FOUND\r\n\r\n";
const GET_ROOT_REQ: &str = "GET / HTTP/1.1";
const DEFAULT_HTML_PATH: &str = "assets/hello.html";
const DEFAULT_NOT_FOUND_PATH: &str = "assets/404.html";
fn get_bind_address(addr: &str, port: &str) -> String {
    format!("{addr}:{port}")
}
fn main() -> Result<(), Box<dyn Error>>{
    let tcp_listener = TcpListener::bind(get_bind_address(LOCALHOST, PORT))?;
    let pool = ThreadPool::build(4)?;
    tcp_listener
    .incoming()
    .for_each(|stream| {
        let stream = stream.unwrap();
        pool.execute(|| {
            let _ = handle_connection(stream);
        })
        
        
    });
    Ok(())
}

fn handle_connection(mut stream: TcpStream) -> Result<(), Box<dyn Error>> {
    let buf_reader = BufReader::new(&mut stream);
    let request_line = buf_reader.lines().next().unwrap()?;
    
   
    let (status_line, filename) = match &request_line[..] {
        GET_ROOT_REQ => (OK_RESPONSE, DEFAULT_HTML_PATH),
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(5));
            (OK_RESPONSE, DEFAULT_HTML_PATH)
        }
        _ => (NOT_FOUND_RESPONSE, DEFAULT_NOT_FOUND_PATH),
    };

    let response = get_html(filename, status_line)?;
    println!("{response}");
    stream.write_all(response.as_bytes()).unwrap();

    Ok(())
}

fn get_html(path: &str, status_line: &str) -> Result<String, Box<dyn Error>> {
    let contents = fs::read_to_string(path)?;
        let length = contents.len();

        let response = format!(
            "{status_line}\r\nContent-Type: text/html; charset=UTF-8\r\nContent-Length: {length}\r\n\r\n{contents}"
        );
        Ok(response)
}
