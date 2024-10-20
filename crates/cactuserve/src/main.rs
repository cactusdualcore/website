use cactuserve_sessions::Auth;
use maud::{html, Markup, DOCTYPE};
use rocket::{
    get,
    http::{Cookie, CookieJar},
    routes,
    time::{Duration, OffsetDateTime},
};

#[rocket::main]
async fn main() -> anyhow::Result<()> {
    cactuserve::users::key_pair();

    let static_file_server = rocket::fs::FileServer::from("web/static");
    let _ = rocket::build()
        .mount("/", routes![index, favicon, requires_auth, create_token])
        .mount("/static", static_file_server)
        .launch()
        .await?;

    Ok(())
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/favicon.ico")]
async fn favicon() -> Option<rocket::fs::NamedFile> {
    rocket::fs::NamedFile::open("web/static/favicon.ico")
        .await
        .ok()
}

#[get("/awesome")]
fn requires_auth(
    auth: Auth<cactuserve::users::Reader, cactuserve::users::Provider>,
) -> Markup {
    let user = auth.token();
    html! {
        (DOCTYPE)
        html {
            head {
                title { "Awesome!" }
            }
            body {
                p {
                    "You're awesome, " b { (user.username()) } "!"
                }
            }
        }
    }
}

#[get("/mkuser/<username>")]
fn create_token(username: &str, cookies: &CookieJar<'_>) -> Markup {
    let user = cactuserve::users::User {
        username: username.to_string(),
        scope: cactuserve::users::Scope::empty(),
    };

    let token = cactuserve_sessions::to_str(
        &cactuserve::users::Writer,
        &user,
        cactuserve::users::key_pair(),
    )
    .unwrap();

    let session_cookie = Cookie::build(("cactuserve-session", token.clone()))
        .expires(OffsetDateTime::now_utc().saturating_add(Duration::days(30)))
        .secure(true)
        .same_site(rocket::http::SameSite::Strict)
        .partitioned(true)
        .build();
    cookies.add(session_cookie);

    html! {
        (DOCTYPE)
        html {
            head {
                title { "Make Token" }
            }
            body {
                p { "Your Token: " pre { "Bearer " (token) } }
            }
        }
    }
}
