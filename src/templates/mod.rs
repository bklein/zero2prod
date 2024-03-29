mod registry;
pub use registry::*;

mod admin;
pub use admin::*;

mod login;
pub use login::*;

mod home;
pub use home::*;

#[cfg(test)]
mod test_helpers;
#[cfg(test)]
use test_helpers::assert_and_get_element;
