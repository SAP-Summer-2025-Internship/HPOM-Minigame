use std::{
    collections::HashMap,
    fs,
    io::{BufReader, prelude::*},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
};

mod user_session;
use user_session::UserSession;

type Sessions = Arc<Mutex<HashMap<String, UserSession>>>;

fn main() {
    // TODO figure out a way to log all information, fly.io volumes?
    let sessions: Sessions = Arc::new(Mutex::new(HashMap::new()));
    // Use 0.0.0.0:8080 on Fly.io, otherwise use 127.0.0.1:7878
    let bind_addr = if std::env::var("FLY_APP_NAME").is_ok() {
        "0.0.0.0:8080"
    } else {
        "127.0.0.1:7878"
    };
    let listener = TcpListener::bind(bind_addr).unwrap();
    
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let sessions = Arc::clone(&sessions);
        thread::spawn(move || {
            handle_connection(stream, sessions);
        });
    }
}

fn handle_connection(mut stream: TcpStream, sessions: Sessions) {
    let buf_reader = BufReader::new(&stream);
    let http_request: Vec<String> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();
    
    let request_line = &http_request[0];

    let (status_line, contents, content_type, set_cookie) = if request_line.starts_with("GET /lib/") {
        // Serve static files (images, etc.)
        let path = request_line.split_whitespace().nth(1).unwrap().trim_start_matches('/');
        match fs::read(path) {
            Ok(bytes) => ("HTTP/1.1 200 OK".to_string(), bytes, get_content_type(path).to_string(), None),
            Err(_) => ("HTTP/1.1 404 NOT FOUND".to_string(), Vec::new(), "text/plain".to_string(), None),
        }
    } else {
        handle_app_request(http_request.join("\n"), sessions)
    };

    let length = contents.len();
    let mut response = format!("{status_line}\r\nContent-Length: {length}\r\nContent-Type: {content_type}\r\n");
    
    if let Some(cookie) = set_cookie {
        response.push_str(&format!("Set-Cookie: {}\r\n", cookie));
    }
    
    response.push_str("\r\n");
    stream.write_all(response.as_bytes()).unwrap();
    stream.write_all(&contents).unwrap();
}

fn handle_app_request(request: String, sessions: Sessions) -> (String, Vec<u8>, String, Option<String>) {
    let request_line = request.lines().next().unwrap();
    println!("Request: {}", request_line);

    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        return ("HTTP/1.1 400 BAD REQUEST".to_string(), Vec::new(), "text/plain".to_string(), None);
    }

    let url_parts: Vec<&str> = parts[1].split('?').collect();
    let query = if url_parts.len() > 1 { Some(url_parts[1]) } else { None };

    // Extract session ID from cookie
    let session_id = extract_session_id(&request).unwrap_or_else(|| generate_session_id());
    println!("[DEBUG] Extracted session ID: {:?}", extract_session_id(&request));
    println!("[DEBUG] Using session ID: {}", session_id);
    
    let set_cookie = if extract_session_id(&request).is_none() {
        println!("[DEBUG] No session found, setting new cookie");
        Some(format!("session_id={}; Path=/", session_id))
    } else {
        println!("[DEBUG] Session found, no new cookie needed");
        None
    };

    let mut sessions_guard = sessions.lock().unwrap();
    
    // Check for restart
    if let Some(query) = &query {
        if query.contains("restart=true") {
            sessions_guard.remove(&session_id);
        }
    }
    
    let session = sessions_guard.entry(session_id.clone()).or_insert(UserSession::new());
    
    // Debug: print session info before update
    println!("[DEBUG] Session ID: {}", session_id);
    println!("[DEBUG] Current page: {}", session.current_page());
    println!("[DEBUG] Button presses: {:?}", session.button_presses());

    // Handle button press from query parameters
    let mut remove_session = false;
    if let Some(query) = query {
        if let Some(button) = parse_button_press(query) {
            println!("[DEBUG] Attempting button press: '{}' from page {}", button, session.current_page());
            // Process the button press
            match session.process_button_press(&button) {
                Ok(next_page) => {
                    println!("Session {}: Button press '{}' validated! Moving to page {}", 
                             session_id, button, next_page);
                    println!("[DEBUG] Updated button presses: {:?}", session.button_presses());
                    if next_page == 9 {
                        remove_session = true;
                    }
                },
                Err(error) => {
                    match error {
                        user_session::ValidationError::InvalidButton(btn, allowed) => {
                            println!("[DEBUG] VALIDATION FAILED: Button '{}' not allowed from page {}. Allowed buttons: {:?}", 
                                     btn, session.current_page(), allowed);
                        },
                        user_session::ValidationError::NoTransitionDefined(page) => {
                            println!("[DEBUG] ERROR: No transitions defined for page {}", page);
                        },
                        user_session::ValidationError::InvalidPage(page) => {
                            println!("[DEBUG] ERROR: Invalid page transition from page {}", page);
                        },
                    }
                    // Don't update session, just serve the current page again
                }
            }
        }
    }

    // Remove session if user has reached page 9
    if remove_session {
        let user_session = sessions_guard.remove(&session_id).unwrap();
        let _doc_string = UserSession::to_doc_string(user_session);
        // TODO implement docstring fly.io volume here
        // Serve page 1 after session removal
        let html = load_page_html(9, &[]);
        return ("HTTP/1.1 200 OK".to_string(), html.into_bytes(), "text/html".to_string(), set_cookie);
    }

    let page_to_serve = session.current_page();
    let html = load_page_html(page_to_serve, session.button_presses());
    
    // Debug: print all sessions after update
    println!("\n[DEBUG] All sessions after update: {{");
    for (sid, sess) in sessions_guard.iter() {
        println!("  {} => Current page: {}, Button presses: {:?}", 
                 sid, sess.current_page(), sess.button_presses());
    }
    println!("}}\n");
    
    ("HTTP/1.1 200 OK".to_string(), html.into_bytes(), "text/html".to_string(), set_cookie)
}

fn extract_session_id(request: &str) -> Option<String> {
    for line in request.lines() {
        if line.to_lowercase().starts_with("cookie:") {
            if let Some(start) = line.find("session_id=") {
                let start = start + "session_id=".len();
                let end = line[start..].find(';').unwrap_or(line.len() - start);
                let session_id = line[start..start + end].to_string();
                println!("[DEBUG] Extracted session_id: {}", session_id);
                return Some(session_id);
            }
        }
    }
    println!("[DEBUG] No session_id found in request");
    None
}

fn generate_session_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    format!("session_{}", timestamp)
}

fn parse_button_press(query: &str) -> Option<String> {
    // Parse query like "button=pm" or "action=pm"
    for param in query.split('&') {
        if let Some((key, value)) = param.split_once('=') {
            if key == "button" || key == "action" {
                return Some(value.to_string());
            }
        }
    }
    None
}

fn load_page_html(page: usize, _button_presses: &[String]) -> String {
    let filename = format!("page{}.html", page);
    match fs::read_to_string(&filename) {
        Ok(html) => html,
        Err(_) => {
            format!("<html><body><h1>Page {} not found</h1></body></html>", page)
        }
    }
}

fn get_content_type(path: &str) -> &str {
    if path.ends_with(".jpg") || path.ends_with(".jpeg") {
        "image/jpeg"
    } else if path.ends_with(".png") {
        "image/png"
    } else if path.ends_with(".gif") {
        "image/gif"
    } else if path.ends_with(".css") {
        "text/css"
    } else if path.ends_with(".js") {
        "application/javascript"
    } else {
        "application/octet-stream"
    }
}
