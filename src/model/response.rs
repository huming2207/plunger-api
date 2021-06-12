use rocket::serde::Serialize;

#[derive(Serialize)]
pub(crate) struct PlungerResponse<T = ()> {
    pub(crate) message: String,
    pub(crate) details: Option<T>
}
