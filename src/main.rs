#![feature(proc_macro_hygiene, decl_macro)]
use rocket::{
    http::Status,
    post,
    response::{self, status::Custom, Responder},
    routes, Request,
};
use rocket_contrib::json::{Json, JsonError};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug)]
struct DeserializeError<'a>(JsonError<'a>);

impl Responder<'_> for DeserializeError<'_> {
    fn respond_to(self, req: &Request) -> response::Result<'static> {
        let responder = match self.0 {
            JsonError::Io(e) => Custom(Status::BadRequest, Json(json!({ "error": e.to_string() }))),

            JsonError::Parse(input, error) => Custom(
                Status::UnprocessableEntity,
                Json(json!({ "error": error.to_string(), "input": input })),
            ),
        };

        responder.respond_to(req)
    }
}

impl<'a> From<JsonError<'a>> for DeserializeError<'a> {
    fn from(je: JsonError) -> DeserializeError {
        DeserializeError(je)
    }
}

//---

#[derive(Deserialize, Serialize, Debug)]
struct Thing {
    important_field: bool,
}

#[post("/things", data = "<thing>", format = "json")]
fn create_thing<'a>(thing: Result<Json<Thing>, JsonError<'a>>) -> Result<(), DeserializeError> {
    let thing = &*thing?;

    if thing.important_field {
        // etc
    }

    Ok(())
}

fn rocket() -> rocket::Rocket {
    rocket::ignite().mount("/", routes![create_thing])
}

fn main() {
    rocket().launch();
}

#[cfg(test)]
mod test {
    use super::rocket;
    use rocket::http::Status;
    use rocket::local::Client;
    use serde_json::json;

    #[test]
    fn test_json_parse_error() {
        let client = Client::new(rocket()).expect("valid rocket instance");
        let mut response = client
            .post("/things")
            .header(rocket::http::ContentType::JSON)
            .body("{}")
            .dispatch();
        assert_eq!(response.status(), Status::UnprocessableEntity);
        assert_eq!(
            response.content_type(),
            Some(rocket::http::ContentType::JSON)
        );
        assert_eq!(
            response.body_string().unwrap(),
            json!({
                "error": "missing field `important_field` at line 1 column 2",
                "input":"{}"
            })
            .to_string()
        );
    }
}
