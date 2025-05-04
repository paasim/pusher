use crate::err::Result;
use crate::err_other;
use std::env::var;

pub fn to_array<const N: usize, V: AsRef<[u8]>>(slice: V) -> Result<[u8; N]> {
    Ok(err_other!(slice.as_ref()[..N].try_into())?)
}

pub fn get_var(var_name: &str) -> Result<String> {
    Ok(err_other!(
        var(var_name),
        "environment variable '{var_name}' missing"
    )?)
}
