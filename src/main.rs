// use rplace::api::{index, game};
// use rocket::{launch, routes};
// mod rplace;

// #[launch]
// fn rocket() -> _ {
//     rocket::build()
//         .mount("/", routes![index])
//         .mount("/game", routes![game])
// }

mod rplace;
fn main() {
    rplace::main();
}