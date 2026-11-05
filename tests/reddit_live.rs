//! Explicit live checks: cargo test --test reddit_live -- --ignored --test-threads=1
use redlib::client::json;

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires live Reddit access"]
async fn public_subreddit_feed_is_readable() {
	let listing = json("/r/rust/hot.json?raw_json=1".into(), false).await.expect("a public subreddit feed should be readable");
	assert_eq!(listing["kind"], "Listing");
	assert!(!listing["data"]["children"].as_array().expect("listing children").is_empty());
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires live Reddit access"]
async fn new_feed_remains_readable_and_sorted_by_time() {
	let feed = json("/r/rust/new.json?raw_json=1&limit=10".into(), false).await.expect("new feed");
	let posts = feed["data"]["children"].as_array().expect("new posts");
	assert!(!posts.is_empty());
	let times: Vec<_> = posts
		.iter()
		.filter(|p| p["data"]["stickied"] != true)
		.map(|p| p["data"]["created_utc"].as_f64().expect("creation time"))
		.collect();
	assert!(times.windows(2).all(|w| w[0] >= w[1]), "new feed is chronological");
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires live Reddit access"]
async fn all_and_popular_feeds_support_new_sort() {
	for subreddit in ["all", "popular"] {
		let feed = json(format!("/r/{subreddit}/new.json?raw_json=1&limit=5"), false).await.expect("global New feed");
		assert!(!feed["data"]["children"].as_array().expect("global feed posts").is_empty());
	}
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires live Reddit access"]
async fn new_listing_pagination_returns_the_next_posts() {
	let first = json("/r/rust/new.json?raw_json=1&limit=5".into(), false).await.expect("first listing page");
	let after = first["data"]["after"].as_str().expect("next page cursor");
	let query = url::form_urlencoded::Serializer::new(String::new())
		.append_pair("after", after)
		.append_pair("raw_json", "1")
		.append_pair("limit", "5")
		.finish();
	let next = json(format!("/r/rust/new.json?{query}"), false).await.expect("next listing page");
	let first_ids: Vec<_> = first["data"]["children"].as_array().unwrap().iter().map(|p| p["data"]["id"].as_str().unwrap()).collect();
	let next_posts = next["data"]["children"].as_array().expect("next posts");
	assert!(!next_posts.is_empty());
	assert!(
		next_posts.iter().all(|p| !first_ids.contains(&p["data"]["id"].as_str().unwrap())),
		"pagination returns different posts"
	);
}
