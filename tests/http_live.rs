//! Explicit browser-facing smoke tests; these require live Reddit access.
use hyper::{body::to_bytes, Client};
use std::{
	net::TcpListener,
	process::{Child, Command, Stdio},
	time::Duration,
};

struct App(Child);
impl Drop for App {
	fn drop(&mut self) {
		let _ = self.0.kill();
		let _ = self.0.wait();
	}
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires live Reddit access"]
async fn rendered_video_sources_are_proxied() {
	let listener = TcpListener::bind("127.0.0.1:0").unwrap();
	let port = listener.local_addr().unwrap().port();
	drop(listener);
	let _app = App(
		Command::new(env!("CARGO_BIN_EXE_redlib"))
			.args(["--address", "127.0.0.1", "--port", &port.to_string()])
			.env("REDLIB_DEFAULT_USE_HLS", "on")
			.stdout(Stdio::null())
			.stderr(Stdio::null())
			.spawn()
			.unwrap(),
	);
	let client = Client::new();
	let base = format!("http://127.0.0.1:{port}");
	tokio::time::timeout(Duration::from_secs(45), async {
		loop {
			if client.get(format!("{base}/settings").parse().unwrap()).await.is_ok() {
				break;
			}
			tokio::time::sleep(Duration::from_millis(100)).await;
		}
	})
	.await
	.expect("app starts");
	let response = client.get(format!("{base}/r/oddlysatisfying/comments/1xofz96").parse().unwrap()).await.unwrap();
	assert_eq!(response.status(), 200);
	let html = String::from_utf8(to_bytes(response.into_body()).await.unwrap().to_vec()).unwrap();
	let source = regex::Regex::new(r#"<source src="([^"]+)""#).unwrap();
	let sources: Vec<_> = source.captures_iter(&html).map(|c| c[1].to_owned()).collect();
	assert!(!sources.is_empty(), "post renders a video");
	assert!(sources.iter().all(|s| s.starts_with('/')), "video sources must stay on this instance: {sources:?}");
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires live Reddit access"]
async fn more_reply_links_preserve_cursor_and_sort() {
	let listener = TcpListener::bind("127.0.0.1:0").unwrap();
	let port = listener.local_addr().unwrap().port();
	drop(listener);
	let _app = App(
		Command::new(env!("CARGO_BIN_EXE_redlib"))
			.args(["--address", "127.0.0.1", "--port", &port.to_string()])
			.stdout(Stdio::null())
			.stderr(Stdio::null())
			.spawn()
			.unwrap(),
	);
	let client = Client::new();
	let base = format!("http://127.0.0.1:{port}");
	tokio::time::timeout(Duration::from_secs(45), async {
		loop {
			if client.get(format!("{base}/settings").parse().unwrap()).await.is_ok() {
				break;
			}
			tokio::time::sleep(Duration::from_millis(100)).await;
		}
	})
	.await
	.expect("app starts");
	let response = client.get(format!("{base}/r/rust/comments/18t5968?sort=confidence").parse().unwrap()).await.unwrap();
	assert_eq!(response.status(), 200);
	let html = String::from_utf8(to_bytes(response.into_body()).await.unwrap().to_vec()).unwrap();
	let link = regex::Regex::new(r#"class="deeper_replies" href="([^"]+)""#).unwrap();
	let links: Vec<_> = link.captures_iter(&html).map(|c| c[1].to_owned()).collect();
	assert!(!links.is_empty(), "fixture thread contains deep replies");
	assert!(
		links.iter().all(|s| s.contains("cursor=") && s.contains("sort=confidence")),
		"continuations must retain their cursor and sort: {links:?}"
	);
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires live Reddit access"]
async fn long_threads_offer_the_next_page_of_comments() {
	let feed = redlib::client::json("/r/AskReddit/top.json?t=day&raw_json=1&limit=5".into(), false)
		.await
		.expect("busy thread feed");
	let original = feed["data"]["children"]
		.as_array()
		.unwrap()
		.iter()
		.find(|p| p["data"]["num_comments"].as_u64().unwrap_or(0) > 300)
		.expect("a busy thread in the live feed");
	let id = original["data"]["id"].as_str().unwrap();
	let listener = TcpListener::bind("127.0.0.1:0").unwrap();
	let port = listener.local_addr().unwrap().port();
	drop(listener);
	let _app = App(
		Command::new(env!("CARGO_BIN_EXE_redlib"))
			.args(["--address", "127.0.0.1", "--port", &port.to_string()])
			.stdout(Stdio::null())
			.stderr(Stdio::null())
			.spawn()
			.unwrap(),
	);
	let client = Client::new();
	let base = format!("http://127.0.0.1:{port}");
	tokio::time::timeout(Duration::from_secs(45), async {
		loop {
			if client.get(format!("{base}/settings").parse().unwrap()).await.is_ok() {
				break;
			}
			tokio::time::sleep(Duration::from_millis(100)).await;
		}
	})
	.await
	.expect("app starts");
	let response = client.get(format!("{base}/r/AskReddit/comments/{id}?sort=confidence").parse().unwrap()).await.unwrap();
	assert_eq!(response.status(), 200);
	let html = String::from_utf8(to_bytes(response.into_body()).await.unwrap().to_vec()).unwrap();
	let next = regex::Regex::new(r#"href="([^"]*cursor=[^"]*)"[^>]*>[^<]*(?:Next|More comments)"#).unwrap();
	let href = next.captures(&html).expect("a long thread offers a next comments page")[1]
		.replace("&amp;", "&")
		.replace("&#38;", "&");
	let response = client
		.get(
			url::Url::parse(&format!("{base}/r/AskReddit/comments/{id}"))
				.unwrap()
				.join(&href)
				.unwrap()
				.as_str()
				.parse()
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(response.status(), 200, "the next page loads");
}
