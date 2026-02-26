use actix_web::{
    HttpRequest, HttpResponse,
    cookie::{Cookie, time::Duration},
    http::header::ContentType,
};

pub async fn login_form(request: HttpRequest) -> HttpResponse {
    let error_message = match request.cookie("_flash") {
        Some(cookie) => {
            format!("<p><i>{}</i></p>", cookie.value())
        }
        None => "".into(),
    };
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .cookie(Cookie::build("_flash", "").max_age(Duration::ZERO).finish())
        .body(format!(
            r#"<!DOCTYPE html>
    <html lang="en">
    <head>
    <meta http-equiv="content-type" content="text/html; charset=utf-8">
    <title>Login</title>
    </head>
    <body>
    {error_message}
    <form action="/login" method="post">
    <label>Username
    <input
    type="text"
    placeholder="Enter Username"
    name="username"
    >
    </label>
    <label>Password
    <input
    type="password"
    placeholder="Enter Password"
    name="password"
    >
    </label>
    <button type="submit">Login</button>
    </form>
    </body>
    </html>"#,
        ))
}
