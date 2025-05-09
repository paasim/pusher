use pusher::err::{Error, Result};
use serde::Serialize;
use std::io;
use std::io::Read;
use tokio::io::AsyncReadExt;
use tokio::net::UnixStream;

const ICON: &str = "push-small.png";

#[derive(Debug)]
pub struct Msg {
    title: String,
    body: String,
}

impl Serialize for Msg {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Serialize)]
        struct MsgRaw<'a> {
            title: &'a str,
            options: MsgOpt<'a>,
        }
        #[derive(Serialize)]
        pub struct MsgOpt<'a> {
            body: &'a str,
            icon: &'static str,
        }
        let raw = MsgRaw {
            title: &self.title,
            options: MsgOpt {
                body: &self.body,
                icon: ICON,
            },
        };
        raw.serialize(serializer)
    }
}

impl Msg {
    /// Read message body from the stream
    pub async fn from_stream(mut stream: UnixStream, title: String) -> Result<Self> {
        let mut body = String::new();
        stream.read_to_string(&mut body).await?;
        Ok(Self { title, body })
    }

    /// Read message body from [io::stdin()]
    pub fn from_stdin(title: String) -> Result<Self> {
        let mut body = String::new();
        io::stdin().read_to_string(&mut body).unwrap();
        Ok(Self { title, body })
    }
}

impl TryFrom<Msg> for Vec<u8> {
    type Error = Error;

    /// Turn the message into JSON and serialize the result as utf-8 bytes.
    fn try_from(msg: Msg) -> Result<Self> {
        Ok(serde_json::to_string(&msg)?.as_bytes().to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_vec_works() {
        let msg = Msg {
            title: String::from("title 1"),
            body: String::from("this is a body"),
        };
        let content = Vec::try_from(msg).unwrap();

        let content_exp1 = format!(
            r#"{{"title":"title 1","options":{{"body":"this is a body","icon":"{ICON}"}}}}"#
        );
        assert!(content == content_exp1.as_bytes())
    }
}
