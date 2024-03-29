use axum::http::HeaderMap;
use rand::{distributions::Alphanumeric, Rng};

/// Generate a random string of a given length
/// # Examples
/// ```
/// use url_shortener::utils::strings::generate_random_string;
/// let rand_str = generate_random_string(5);
/// assert_eq!(rand_str.len(), 5);
/// ```
pub fn generate_random_string(length: usize) -> String {
    let rand_string: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .filter(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        .take(length)
        .map(char::from)
        .collect();

    rand_string
}

pub fn get_host_header(headers: &HeaderMap, add_proto: bool) -> String {
    let string = headers
        .get("host")
        .expect("host header not found")
        .to_str()
        .expect("host header couldn't be converted to string");

    match add_proto {
        false => string.to_string(),
        true if string.contains("localhost") => "http://localhost:8080".to_string(),
        true if string.starts_with("http") => string.to_string(),
        true => format!("https://{}", string),
    }
}
