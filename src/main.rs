#[macro_use] extern crate rocket;

mod routes;
mod massprod;
mod model;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[launch]
fn rocket() -> _ {
    dotenv::dotenv().ok();

    rocket::build().mount("/", routes![index])
}