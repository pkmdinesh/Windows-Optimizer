use crate::engine::OptimizationOrchestrator;
use crate::hardware::models::HardwareProfile;
use crate::modules::{CleanerManager, DebloatManager, StartupManager};
use std::io::Cursor;
use std::path::Path;
use std::process::Command;
use std::sync::Arc;
use tiny_http::{Header, Method, Response, Server, StatusCode};

pub fn launch_gui_window(profile: HardwareProfile) {
    let addr = "127.0.0.1:8765";
    let target_url = format!("http://{}", addr);

    // Launch standalone application window via Chrome or Edge or default browser
    let chrome_path = "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe";
    let edge_paths = [
        "C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe",
        "C:\\Program Files\\Microsoft\\Edge\\Application\\msedge.exe",
    ];

    let mut launched = false;

    if Path::new(chrome_path).exists() {
        if Command::new(chrome_path)
            .args([
                &format!("--app={}", target_url),
                "--window-size=1200,850",
                "--disable-features=Translate",
            ])
            .spawn()
            .is_ok()
        {
            launched = true;
        }
    }

    if !launched {
        for edge in edge_paths {
            if Path::new(edge).exists() {
                if Command::new(edge)
                    .args([
                        &format!("--app={}", target_url),
                        "--window-size=1200,850",
                    ])
                    .spawn()
                    .is_ok()
                {
                    launched = true;
                    break;
                }
            }
        }
    }

    if !launched {
        let _ = Command::new("cmd").args(["/c", "start", &target_url]).spawn();
    }

    // Run the server on the current thread
    start_web_server(profile);
}

pub fn start_web_server(profile: HardwareProfile) {
    let orchestrator = Arc::new(OptimizationOrchestrator::new(profile.clone()));
    let addr = "127.0.0.1:8765";
    let server = match Server::http(addr) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to bind web server to {}: {}", addr, e);
            return;
        }
    };

    println!("Local GUI server running at http://{}", addr);

    let html_content = include_str!("../assets/dashboard.html");

    for request in server.incoming_requests() {
        let url = request.url().to_string();
        let method = request.method().clone();

        let json_header = Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap();
        let html_header = Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap();

        if (url == "/" || url.starts_with("/?")) && method == Method::Get {
            let response = Response::new(
                StatusCode(200),
                vec![html_header],
                Cursor::new(html_content),
                Some(html_content.len()),
                None,
            );
            let _ = request.respond(response);
            continue;
        }

        if url == "/api/profile" && method == Method::Get {
            let json = serde_json::to_string(&profile).unwrap_or_else(|_| "{}".into());
            let response = Response::new(
                StatusCode(200),
                vec![json_header],
                Cursor::new(json.clone()),
                Some(json.len()),
                None,
            );
            let _ = request.respond(response);
            continue;
        }

        if url == "/api/tweaks" && method == Method::Get {
            let tweaks = orchestrator.get_tweak_infos();
            let json = serde_json::to_string(&tweaks).unwrap_or_else(|_| "[]".into());
            let response = Response::new(
                StatusCode(200),
                vec![json_header],
                Cursor::new(json.clone()),
                Some(json.len()),
                None,
            );
            let _ = request.respond(response);
            continue;
        }

        if url.starts_with("/api/apply-preset") && method == Method::Post {
            let res = orchestrator.apply_safe_preset();
            let applied = res.unwrap_or(0);
            let json = format!(r#"{{"status": "ok", "applied": {}}}"#, applied);
            let response = Response::new(
                StatusCode(200),
                vec![json_header],
                Cursor::new(json.clone()),
                Some(json.len()),
                None,
            );
            let _ = request.respond(response);
            continue;
        }

        if url.starts_with("/api/apply-tweak") && method == Method::Post {
            let tweak_id = url.split("id=").nth(1).unwrap_or("");
            let result = orchestrator.apply_tweak_by_id(tweak_id);
            let status_code = if result.is_ok() { 200 } else { 400 };
            let json = format!(r#"{{"status": "{}"}}"#, if result.is_ok() { "ok" } else { "error" });
            let response = Response::new(
                StatusCode(status_code),
                vec![json_header],
                Cursor::new(json.clone()),
                Some(json.len()),
                None,
            );
            let _ = request.respond(response);
            continue;
        }

        if url.starts_with("/api/revert-tweak") && method == Method::Post {
            let tweak_id = url.split("id=").nth(1).unwrap_or("");
            let result = orchestrator.revert_tweak_by_id(tweak_id);
            let status_code = if result.is_ok() { 200 } else { 400 };
            let json = format!(r#"{{"status": "{}"}}"#, if result.is_ok() { "ok" } else { "error" });
            let response = Response::new(
                StatusCode(status_code),
                vec![json_header],
                Cursor::new(json.clone()),
                Some(json.len()),
                None,
            );
            let _ = request.respond(response);
            continue;
        }

        if url.starts_with("/api/rollback-all") && method == Method::Post {
            let count = orchestrator.rollback_all().unwrap_or(0);
            let json = format!(r#"{{"status": "ok", "count": {}}}"#, count);
            let response = Response::new(
                StatusCode(200),
                vec![json_header],
                Cursor::new(json.clone()),
                Some(json.len()),
                None,
            );
            let _ = request.respond(response);
            continue;
        }

        if url == "/api/bloatware" && method == Method::Get {
            let apps = DebloatManager::scan_installed_bloatware();
            let json = serde_json::to_string(&apps).unwrap_or_else(|_| "[]".into());
            let response = Response::new(
                StatusCode(200),
                vec![json_header],
                Cursor::new(json.clone()),
                Some(json.len()),
                None,
            );
            let _ = request.respond(response);
            continue;
        }

        if url.starts_with("/api/purge-bloatware") && method == Method::Post {
            let results = DebloatManager::purge_all_safe_bloatware();
            let json = serde_json::to_string(&serde_json::json!({ "removed": results })).unwrap();
            let response = Response::new(
                StatusCode(200),
                vec![json_header],
                Cursor::new(json.clone()),
                Some(json.len()),
                None,
            );
            let _ = request.respond(response);
            continue;
        }

        if url == "/api/startup" && method == Method::Get {
            let items = StartupManager::get_registered_startup_programs();
            let json = serde_json::to_string(&items).unwrap_or_else(|_| "[]".into());
            let response = Response::new(
                StatusCode(200),
                vec![json_header],
                Cursor::new(json.clone()),
                Some(json.len()),
                None,
            );
            let _ = request.respond(response);
            continue;
        }

        // --- NEW JUNK CLEANER & RAM BOOST APIS ---
        if url == "/api/junk-scan" && method == Method::Get {
            let junk = CleanerManager::scan_junk();
            let json = serde_json::to_string(&junk).unwrap_or_else(|_| "[]".into());
            let response = Response::new(
                StatusCode(200),
                vec![json_header],
                Cursor::new(json.clone()),
                Some(json.len()),
                None,
            );
            let _ = request.respond(response);
            continue;
        }

        if url.starts_with("/api/junk-clean") && method == Method::Post {
            let res = CleanerManager::clean_all_junk();
            let json = serde_json::to_string(&res).unwrap_or_else(|_| "{}".into());
            let response = Response::new(
                StatusCode(200),
                vec![json_header],
                Cursor::new(json.clone()),
                Some(json.len()),
                None,
            );
            let _ = request.respond(response);
            continue;
        }

        if url.starts_with("/api/boost-memory") && method == Method::Post {
            let res = CleanerManager::boost_memory_and_network();
            let json = serde_json::to_string(&res).unwrap_or_else(|_| "{}".into());
            let response = Response::new(
                StatusCode(200),
                vec![json_header],
                Cursor::new(json.clone()),
                Some(json.len()),
                None,
            );
            let _ = request.respond(response);
            continue;
        }

        // Default 404
        let _ = request.respond(Response::empty(StatusCode(404)));
    }
}
