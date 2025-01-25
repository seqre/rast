//! The C2 server part of the Rast project.

pub mod c2;
pub mod messages;

pub use c2::RastC2;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
