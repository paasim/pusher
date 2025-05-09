use crate::err::Result;
use crate::err_other;
use std::env::var;

/// Transform slice into array of size `N`, discarding all the extra elements.
pub fn to_array<const N: usize, V: AsRef<[u8]>>(slice: V) -> Result<[u8; N]> {
    Ok(err_other!(slice.as_ref()[..N].try_into())?)
}

/// Return an environment variable, returning a readable [crate::err::Error] if it does not exist
pub fn get_var(var_name: &str) -> Result<String> {
    Ok(err_other!(
        var(var_name),
        "environment variable '{var_name}' missing"
    )?)
}
