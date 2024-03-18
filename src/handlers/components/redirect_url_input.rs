use crate::utils::strings::generate_random_string;

pub async fn get() -> impl axum::response::IntoResponse {
    RedirectURLInput {
        shortened_url: generate_random_string(4),
    }
}

#[derive(askama::Template)]
#[template(path = "components/redirect_url_input.html")]
struct RedirectURLInput {
    shortened_url: String,
}
