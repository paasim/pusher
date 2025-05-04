use msg::Msg;
use pusher::err::Res;
use pusher::utils::to_array;
use pusher::{base64::base64url_decode, utils::get_var};
use req::{send_notifications, VapidConfig};

mod msg;
mod req;

fn get_conf() -> Res<([u8; 16], String, VapidConfig, Vec<u8>)> {
    let encryption_key = get_var("DATABASE_ENCRYPTION_KEY")
        .and_then(base64url_decode)
        .and_then(to_array)?;
    let database_path = get_var("DATABASE_PATH")?;
    let vapid = VapidConfig::from_env()?;
    let content = Msg::read().and_then(Vec::try_from)?;
    Ok((encryption_key, database_path, vapid, content))
}

fn main() -> Res<()> {
    let (encryption_key, database_path, vapid, content) = get_conf().unwrap_or_else(|e| {
        eprintln!("{}", e);
        std::process::exit(1)
    });

    let res = send_notifications(&database_path, &vapid, &content, 10, encryption_key)?;
    for (url, status_code, body) in res {
        println!("Push to {}", url);
        println!("  with status code {}", status_code);
        match body.as_ref().map(|s| s.as_str()) {
            Ok("") => {}
            Ok(s) => println!("  {}", s),
            Err(e) => eprintln!("  and non-renderable response {}", e),
        }
    }
    Ok(())
}
