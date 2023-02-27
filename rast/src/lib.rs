//! The Rast project commonly used functionalities.

pub mod messages;
pub mod protocols;
pub mod settings;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
