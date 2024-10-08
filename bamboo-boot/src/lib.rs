//! [bamboo-boot](https://go-bamboo.github.io/docs/plugins/bamboo-boot)
pub mod app;
pub mod builder;
pub mod plugin;
mod component;
pub mod sync;
pub mod time;

pub use tokio::{main, select, spawn};
pub use async_trait::async_trait;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
