use pusher::err::{Error, Result};
use serde::Serialize;
use std::env::args;
use std::io;
use std::io::Read;

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
                icon: "push-small.png",
            },
        };
        raw.serialize(serializer)
    }
}

impl Msg {
    pub fn read() -> Result<Self> {
        let mut args = args();
        let progname = args.next().ok_or("invalid args")?;
        let title = match args.next().as_deref() {
            Some("--title") => args.next().ok_or("title expected after --title")?,
            _ => Err(format!("usage: {} --title title", progname))?,
        };
        let mut stdin = io::stdin();
        let mut body = String::new();
        stdin.read_to_string(&mut body).unwrap();
        Ok(Msg { title, body })
    }
}

impl TryFrom<Msg> for Vec<u8> {
    type Error = Error;

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

        let content_exp1 =
            r#"{"title":"title 1","options":{"body":"this is a body","icon":"push-small.png"}}"#
                .as_bytes();
        assert!(content == content_exp1)
    }
}
