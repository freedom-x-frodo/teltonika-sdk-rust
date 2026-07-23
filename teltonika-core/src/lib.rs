/*
what is required to be configured in order to access the device?
1) user name and password
2) device type and device ip address


auth type: Session


init ->
    DeviceType::RUTX50
    DeviceIpAddress::some address
    Credentials::new(username, password)
*/
pub mod config;
pub mod device;
pub mod error;

pub use error::{Result, TeltonikaError};

pub fn add(left: u64, right: u64) -> u64 {
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
