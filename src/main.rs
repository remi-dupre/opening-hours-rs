extern crate pest;
#[macro_use]
extern crate pest_derive;
extern crate chrono;

pub mod parser;
pub mod time_domain;
pub mod time_selector;

fn main() {
    parser::parse(
        r#"2020-2050Apr28week01-23,30:Mo-Sa 08:00-13:00,14:00-17:00 unknown "not on bad weather days!""#,
    )
    .map_err(|err| eprintln!("{}", err.description))
    .map(|res| println!("{:#?}", res))
    .ok();
}
