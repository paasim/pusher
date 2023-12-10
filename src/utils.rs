use crate::err::{PusherError, Res};
use std::env::var;

pub fn to_array<const N: usize, V: AsRef<[u8]>>(slice: V) -> Res<[u8; N]> {
    Ok(slice.as_ref()[..N].try_into()?)
}

pub fn get_var(var_name: &str) -> Res<String> {
    var(var_name)
        .map_err(|_| PusherError::Other(format!("environment variable '{}' missing", var_name)))
}
