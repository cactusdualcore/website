use base64::Engine;
use maud::{html, Markup, DOCTYPE};

use rocket::fs::NamedFile;
use rocket::request::{FromRequest, Outcome};
use rocket::{fs::FileServer, get, launch, response::Responder, routes, Response};
use rocket::{http::*, Request};

use web_succulents::{scopes::Scope, sessions::Session};

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/favicon.ico")]
async fn favicon() -> Option<NamedFile> {
    rocket::fs::NamedFile::open("web/static/favicon.ico")
        .await
        .ok()
}

#[launch]
fn rocket() -> _ {
    let static_file_server = FileServer::from("web/static");
    rocket::build()
        .mount("/", routes![index, login, awesome, favicon])
        .mount("/static", static_file_server)
}

mod header_names {
    pub const TOKEN: &str = "CactusDualcore-Token";
}

#[derive(Debug)]
struct Login<R> {
    inner: R,
    encoded_token: String,
}

impl<'r, 'o: 'r, R: Responder<'r, 'o>> Responder<'r, 'o> for Login<R> {
    fn respond_to(self, req: &'r Request<'_>) -> rocket::response::Result<'o> {
        Response::build_from(self.inner.respond_to(req)?)
            .header(Header::new(header_names::TOKEN, self.encoded_token))
            .ok()
    }
}

#[get("/login/<username>")]
fn login(username: &str) -> Login<Markup> {
    let mut session = Session::new_empty();
    session.scopes |= Scope::Admin;
    session.username = if username.is_empty() {
        None
    } else {
        Some(username.to_owned())
    };

    let token = postcard::to_vec::<_, 64>(&session).unwrap();
    let encoded_token = base64::prelude::BASE64_URL_SAFE.encode(&token);

    let body = html! {
        (DOCTYPE)
        html {
            head {
                meta charset="utf-8";
                title { "Login" }
            }
            body {
                p { "Hello, " (username) }
                p { (format!("{:?}", token)) }
                button #net { "Fetch!" }
            }
            script src="/static/tiny.js" {}
        }
    };

    Login {
        inner: body,
        encoded_token,
    }
}

#[derive(Debug)]
pub struct WithSession(Session);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for WithSession {
    type Error = ();

    async fn from_request<'a>(req: &'r Request<'a>) -> Outcome<Self, Self::Error> {
        let Some(token) = req.headers().get_one(header_names::TOKEN) else {
            return Outcome::Error((Status::Unauthorized, ()));
        };

        match base64::prelude::BASE64_STANDARD.decode(token) {
            Ok(blob) => match postcard::from_bytes::<Session>(&blob).ok() {
                Some(session) => Outcome::Success(WithSession(session)),
                None => Outcome::Error((Status::Unauthorized, ())),
            },
            Err(_) => Outcome::Error((Status::BadRequest, ())),
        }
    }
}

#[get("/awesome")]
fn awesome(session: WithSession) -> Markup {
    let session = session.0;

    eprintln!("Awesome! {:?}", session);

    html! {
        (DOCTYPE)
        html {
            head {
                meta charset="utf-8"
                title { "Awesome!" }
            }
            body {
                p { b { "Du bist " (session.username.as_deref().unwrap_or("N/A")) "!" } }
                @if session.scopes.contains(Scope::Admin) {
                    p { "Du bist Administrator" }
                }
            }
        }
    }
}
