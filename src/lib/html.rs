use askama::Template;
use axum::{response::{IntoResponse, Response, Html}, http::StatusCode};

#[derive(Template)]
#[template(path = "home.html")]
pub struct HomepageTemplate;

pub struct HtmlTemplate<T>(pub T);
impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),

            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed To Template HTML: {}", e),
            )
                .into_response(),
        }
    }
}
