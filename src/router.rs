use std::{fs, io::prelude::*, net::TcpStream, thread, time::Duration};

const ERROR_ROUTE: &str = "/error";

enum Response {
    BadRequest,
    NotFound,
    Ok(&'static str),
}

pub struct Router {
    request_line: Option<String>,
}

impl Router {
    pub fn new(request_line: Option<String>) -> Self {
        Self { request_line }
    }

    /// Sends an HTTP response with an accompanied HTML file
    pub fn respond(&self, stream: &mut TcpStream) {
        if self.request_line.is_none() {
            return Self::send_response(stream, Response::BadRequest);
        }

        let route = Self::get_route(&self.request_line);

        let response = match route.as_str() {
            "/" => Response::Ok("index.html"),
            "/sleep" => {
                // Simulate a slow response
                thread::sleep(Duration::from_secs(5));
                Response::Ok("index.html")
            }
            _ => Response::NotFound,
        };

        Self::send_response(stream, response)
    }

    /// Extracts a route from a request_line
    ///
    /// Request line is a string in the format of "REQUEST_TYPE ROUTE HTTP_VERSION"
    fn get_route(request_line: &Option<String>) -> String {
        let error_val = String::from(ERROR_ROUTE);
        let request = request_line.as_ref().unwrap_or(&error_val);

        // Request will be provided in the following format:
        // "GET / HTTP/1.1" -> If we split by spaces ' ', the middle point is the
        // requested route!
        let split = request.split(' ').collect::<Vec<&str>>();

        if split.len() < 2 {
            return error_val;
        }

        String::from(split[1])
    }

    /// Writes to stream with an HTTP response
    fn send_response(stream: &mut TcpStream, response: Response) {
        let (status_line, filename) = Self::prepare_response(response);
        let contents = fs::read_to_string(filename).unwrap();
        let length = contents.len();

        let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

        stream.write_all(response.as_bytes()).unwrap()
    }

    /// Prepares a return tuple with HTTP response code and an html file
    ///
    /// HTML file can be changed for OK responses
    fn prepare_response(response: Response) -> (&'static str, &'static str) {
        match response {
            Response::BadRequest => ("HTTP/1.1 400 BAD REQUEST", "error.html"),
            Response::NotFound => ("HTTP/1.1 404 NOT FOUND", "error.html"),
            Response::Ok(filename) => ("HTTP/1.1 200 OK", filename),
        }
    }
}

impl Drop for Router {
    fn drop(&mut self) {
        self.request_line = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::router::Router;

    #[test]
    fn prepared_response_ok() {
        let filename = "file";
        let (code, file) = Router::prepare_response(Response::Ok(filename));

        assert_eq!(filename, file);
        assert!(code.contains("200 OK"));
    }

    #[test]
    fn prepared_response_not_found() {
        let (code, file) = Router::prepare_response(Response::NotFound);

        assert_eq!(file, "error.html");
        assert!(code.contains("404 NOT FOUND"));
    }

    #[test]
    fn prepared_response_bad_request() {
        let (code, file) = Router::prepare_response(Response::BadRequest);

        assert_eq!(file, "error.html");
        assert!(code.contains("400 BAD REQUEST"));
    }

    #[test]
    fn get_router_error() {
        let request_line: Option<String> = None;
        let route = Router::get_route(&request_line);

        assert_eq!(route.as_str(), ERROR_ROUTE);
    }

    #[test]
    fn get_router_ok() {
        let http_response = "GET / HTTP/1.1";
        let request_line = Some(String::from(http_response));
        let route = Router::get_route(&request_line);

        assert_eq!(route.as_str(), "/");
    }
}
