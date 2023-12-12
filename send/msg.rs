use pusher::err::PusherError;
use serde::Serialize;
use std::env;

#[derive(Debug)]
pub struct Msg {
    title: String,
    body: String,
}

impl Serialize for Msg {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
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

impl TryFrom<env::Args> for Msg {
    type Error = PusherError;

    fn try_from(mut args: env::Args) -> Result<Self, Self::Error> {
        let mut title = None;
        let mut body = None;
        args.next();
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--title" => {
                    title = Some(args.next().ok_or("title expected after --title")?);
                }
                "--body" => {
                    body = Some(args.next().ok_or("body expected after --body")?);
                }
                s => Err(format!("saw {}, expected --title or --body", s))?,
            }
        }
        Ok(Msg {
            title: title.ok_or("title missing")?,
            body: body.ok_or("body missing")?,
        })
    }
}

impl TryFrom<Msg> for Vec<u8> {
    type Error = PusherError;

    fn try_from(msg: Msg) -> Result<Self, Self::Error> {
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
        assert!(&content == content_exp1)
    }
}
