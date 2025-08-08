use std::sync::atomic::{AtomicUsize, Ordering};
fn try_write_session_to_csv(
    session_id: &str,
    role: &str,
    qtype: &str,
    team_size: &str,
    role_pref: &str,
    hpom_live: &str,
    richard_cai: &str,
    doc_string: &str,
) {
    let csv_path = "/data/data.csv";
    if !std::path::Path::new(csv_path).exists() {
        println!("[DEBUG] Volume not attached or /data/data.csv does not exist. Skipping CSV write.");
        return;
    }
    println!("[DEBUG] Attempting to write to CSV at {}", csv_path);
    if !std::path::Path::new("/data").exists() {
        println!("[DEBUG] /data directory does not exist!");
    } else {
        println!("[DEBUG] /data directory exists.");
    }
    let mut add_header = false;
    if let Ok(metadata) = std::fs::metadata(csv_path) {
        if metadata.len() == 0 {
            add_header = true;
        }
    }
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(csv_path)
        .unwrap_or_else(|e| {
            println!("[DEBUG] Failed to open {}: {}", csv_path, e);
            panic!("[DEBUG] Could not open CSV file");
        });
    if add_header {
        let header = "session_id,role,question_type,team_size,role_pref,hpom_live,richard_cai,doc_string\n";
        use std::io::Write;
        if let Err(e) = file.write_all(header.as_bytes()) {
            println!("[DEBUG] Failed to write header to {}: {}", csv_path, e);
        } else {
            println!("[DEBUG] Wrote header to {}", csv_path);
        }
    }
    let row = format!(
        "{},{},{},{},{},{},{},\"{}\"\n",
        session_id,
        role,
        qtype,
        team_size,
        role_pref,
        hpom_live,
        richard_cai,
        doc_string
    );
    use std::io::Write;
    if let Err(e) = file.write_all(row.as_bytes()) {
        println!("[DEBUG] Failed to write row to {}: {}", csv_path, e);
    } else {
        println!("[DEBUG] Successfully wrote row to {}", csv_path);
    }
}
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
    const MAX_THREADS: usize = 64;
    static THREAD_COUNT: AtomicUsize = AtomicUsize::new(0);
    // Use 0.0.0.0:8080 on Fly.io, otherwise use 127.0.0.1:7878
    let bind_addr = if std::env::var("FLY_APP_NAME").is_ok() {
        "0.0.0.0:8080"
    } else {
        "127.0.0.1:7878"
    };
    let listener = TcpListener::bind(bind_addr).unwrap();
    
    for stream in listener.incoming() {
        let stream = match stream {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[ERROR] Failed to accept connection: {}", e);
                continue;
            }
        };
        if THREAD_COUNT.load(Ordering::SeqCst) >= MAX_THREADS {
            eprintln!("[WARN] Max concurrent connections reached ({}). Dropping connection.", MAX_THREADS);
            // Optionally: stream.shutdown(Shutdown::Both).ok();
            continue;
        }
        THREAD_COUNT.fetch_add(1, Ordering::SeqCst);
        let sessions = Arc::clone(&sessions);
        thread::spawn(move || {
            handle_connection(stream, sessions);
            THREAD_COUNT.fetch_sub(1, Ordering::SeqCst);
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

    // Endpoint to clear the CSV data
    if parts[1].starts_with("/clear-data") {
        let csv_path = "/data/data.csv";
        let result = std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(csv_path)
            .and_then(|mut file| file.write_all(b"session_id,role,question_type,team_size,role_pref,hpom_live,richard_cai,doc_string\n"));
        let html = match result {
            Ok(_) => "<html><body><h2>CSV data cleared.</h2></body></html>".to_string(),
            Err(_) => "<html><body><h2>Failed to clear CSV data (file not found or volume not attached).</h2></body></html>".to_string(),
        };
        return ("HTTP/1.1 200 OK".to_string(), html.into_bytes(), "text/html".to_string(), None);
    }

    // Pretty CSV view endpoint
    if parts[1].starts_with("/view-data") {
        let csv_path = "/data/data.csv";
        let html = match std::fs::read_to_string(csv_path) {
            Ok(csv) => csv_to_html_table(&csv),
            Err(_) => "<html><body><h2>CSV file not found or volume not attached.</h2></body></html>".to_string(),
        };
        return ("HTTP/1.1 200 OK".to_string(), html.into_bytes(), "text/html".to_string(), None);
    }

    let url_parts: Vec<&str> = parts[1].split('?').collect();
    let query = if url_parts.len() > 1 { Some(url_parts[1]) } else { None };

    // Only allow /, /pageN, /view-data, /clear-data, and static files
    let allowed = parts[1] == "/"
        || parts[1].starts_with("/page")
        || parts[1].starts_with("/view-data")
        || parts[1].starts_with("/clear-data")
        || parts[1].starts_with("/lib/");
    if !allowed {
        let html = std::fs::read_to_string("404.html").unwrap_or_else(|_| "<html><body><h1>404 Not Found</h1></body></html>".to_string());
        return ("HTTP/1.1 404 NOT FOUND".to_string(), html.into_bytes(), "text/html".to_string(), None);
    }
fn csv_to_html_table(csv: &str) -> String {
    let mut lines = csv.lines();
    let header = lines.next();
    let mut html = String::from("<html><head><title>Survey Data</title><style>table{border-collapse:collapse;}th,td{border:1px solid #ccc;padding:6px;}th{background:#f0f0f0;}</style></head><body><h2>Survey Data</h2><table>");
    if let Some(h) = header {
        html.push_str("<tr>");
        for col in h.split(',') {
            html.push_str(&format!("<th>{}</th>", html_escape(col)));
        }
        html.push_str("</tr>");
    }
    for line in lines {
        html.push_str("<tr>");
        let mut in_quotes = false;
        let mut cell = String::new();
        for c in line.chars() {
            match c {
                '"' => in_quotes = !in_quotes,
                ',' if !in_quotes => {
                    html.push_str(&format!("<td>{}</td>", html_escape(&cell)));
                    cell.clear();
                },
                _ => cell.push(c),
            }
        }
        html.push_str(&format!("<td>{}</td>", html_escape(&cell)));
        html.push_str("</tr>");
    }
    html.push_str("</table></body></html>");
    html
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

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
        let bp = user_session.button_presses();
        // Extract role, question_type, answers as in to_doc_string
        let role = bp.get(1).map(|s| match s.as_str() {
            "pm" => "Product Manager",
            "ux" => "UX Designer",
            "engi" => "Engineer",
            "dm" => "Deveveloper Manager",
            _ => s.as_str(),
        }).unwrap_or("");
        let qtype = bp.get(2).map(|s| match s.as_str() {
            "mc" => "Multiple Choice",
            "tf" => "True/False",
            _ => s.as_str(),
        }).unwrap_or("");
        // MC answers
        let mut team_size = "";
        let mut role_pref = "";
        if qtype == "Multiple Choice" {
            if let Some(ans) = bp.get(3) {
                team_size = match ans.as_str() {
                    "4a" => "3-5 people",
                    "4b" => "6-8 people",
                    "4c" => "9-12 people",
                    "4d" => "13-15 people",
                    _ => "(unknown)",
                };
            }
            if let Some(ans) = bp.get(4) {
                role_pref = match ans.as_str() {
                    "6a" => "Product Manager",
                    "6b" => "Developer Manager",
                    "6c" => "Engineer",
                    "6d" => "UX Designer",
                    _ => "(unknown)",
                };
            }
        }
        // TF answers
        let mut hpom_live = "";
        let mut richard_cai = "";
        if qtype == "True/False" {
            if let Some(ans) = bp.get(3) {
                hpom_live = match ans.as_str() {
                    "5t" => "True",
                    "5f" => "False",
                    _ => "(unknown)",
                };
            }
            if let Some(ans) = bp.get(4) {
                richard_cai = match ans.as_str() {
                    "7t" => "True",
                    "7f" => "False",
                    _ => "(unknown)",
                };
            }
        }
        let doc_string = UserSession::to_doc_string(user_session.clone()).replace('\n', "\\n").replace('"', "'");
        try_write_session_to_csv(
            &session_id,
            role,
            qtype,
            team_size,
            role_pref,
            hpom_live,
            richard_cai,
            &doc_string,
        );
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
