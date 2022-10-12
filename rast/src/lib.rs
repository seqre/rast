pub mod protocols;
pub mod settings;

// pub enum Msg {}
pub type Msg = String;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
