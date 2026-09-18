use core::time::Duration;

use std::{
	io::{Read, Write},
	net::TcpListener,
	thread::JoinHandle,
};

pub struct Server {
	pub url: String,
	thread: JoinHandle<Vec<String>>,
}

impl Server {
	pub fn new(replies: Vec<Option<String>>) -> Self {
		let listener = TcpListener::bind("127.0.0.1:0").unwrap();
		listener.set_nonblocking(true).unwrap();
		let url = std::format!("http://{}", listener.local_addr().unwrap());
		let thread = std::thread::spawn(move || {
			let deadline = std::time::Instant::now() + Duration::from_secs(3);
			let mut requests = Vec::new();
			let mut last_request = None;
			while std::time::Instant::now() < deadline {
				match listener.accept() {
					Ok((mut stream, _)) => {
						stream.set_nonblocking(false).unwrap();
						stream.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
						let mut bytes = Vec::new();
						let mut buffer = [0; 4096];
						loop {
							let n = stream.read(&mut buffer).unwrap_or(0);
							if n == 0 {
								break;
							}
							bytes.extend_from_slice(&buffer[..n]);
							if let Some(end) = bytes.windows(4).position(|v| v == b"\r\n\r\n") {
								let head = String::from_utf8_lossy(&bytes[..end]);
								let length = head
									.lines()
									.find_map(|line| {
										line.to_lowercase()
											.strip_prefix("content-length: ")
											.and_then(|v| v.parse::<usize>().ok())
									})
									.unwrap_or(0);
								if bytes.len() >= end + 4 + length {
									break;
								}
							}
						}
						let index = requests.len();
						requests.push(String::from_utf8(bytes).unwrap());
						if let Some(Some(reply)) = replies.get(index) {
							let _ = stream.write_all(reply.as_bytes());
						}
						last_request = Some(std::time::Instant::now());
					},
					Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
						std::thread::sleep(Duration::from_millis(2))
					},
					Err(e) => core::panic!("accept: {e}"),
				}
				if last_request.is_some_and(|last| last.elapsed() > Duration::from_millis(100)) {
					break;
				}
			}
			requests
		});
		Self { url, thread }
	}

	pub fn finish(self) -> Vec<String> {
		self.thread.join().unwrap()
	}
}

#[allow(dead_code)]
pub fn json(status: u16, body: &serde_json::Value) -> Option<String> {
	let body = body.to_string();
	Some(std::format!(
		"HTTP/1.1 {status} Response\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
		body.len()
	))
}

#[allow(dead_code)]
pub fn problem(code: &str, recovery: serde_json::Value) -> serde_json::Value {
	let registry: serde_json::Value = serde_json::from_str(core::include_str!(
		"../../src/problem/registry_snapshot.json"
	))
	.unwrap();
	let row = registry["codes"].as_array().unwrap().iter().find(|row| row["code"] == code).unwrap();
	let mut body = serde_json::json!({"type":row["urn"],"title":row["title"],"status":row["http_status"],"code":code,"request_id":"8051f907-4eeb-4ca9-bd84-b447ea65a68c","recovery":recovery});
	if let Some(detail) = registry["public_details"][code].as_str() {
		body["detail"] = detail.into();
	}

	body
}
