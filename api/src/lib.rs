#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;
extern crate bincode;

pub mod client;
pub mod common;
pub mod error;
pub mod server;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
