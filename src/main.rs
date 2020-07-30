use argon2::{self, Config};
use image::{
    imageops::{self, FilterType},
    png, ColorType, RgbImage,
};
use serenity::{
    http::AttachmentType,
    model::{channel::Message, gateway::Ready},
    prelude::*,
    Result,
};
use std::{borrow::Cow, env, process::Command};
use tracing::*;
use tracing_subscriber::fmt::init;

struct Handler;

impl EventHandler for Handler {
    fn message(&self, ctx: Context, msg: Message) {
        use std::{process::Stdio, time::Duration};
        use wait_timeout::ChildExt;

        if let Some("!ferris") = msg.content.split_whitespace().next() {
            if let Err(why) = msg.channel_id.send_message(&ctx.http, |m| {
                use serenity::utils::MessageBuilder;

                let mut des = MessageBuilder::new();

                des.push(msg.author.mention());

                let code = msg.content[7..].replace("```", "");

                let mut child = Command::new("./jit.sh")
                    .args(&[
                        "jit_naked",
                        &code,
                    ])
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                    .unwrap();

                let timeout = Duration::from_secs(
                    env::var("TIMEOUT")
                        .map(|timeout| timeout.parse().unwrap_or(5))
                        .unwrap_or(5),
                );

                match child.wait_timeout(timeout).unwrap() {
                    Some(_) => {
                        let out = child.wait_with_output().unwrap();

                        let res = format!(
                            "{}\n{}",
                            String::from_utf8_lossy(&out.stdout),
                            String::from_utf8_lossy(&out.stderr)
                        );

                        if out.status.success() {
                            let mut buf = Vec::new();
                            {
                                use rustfmt_nightly::{Config, Edition, EmitMode, Input, Session};

                                let mut config = Config::default();
                                config.set().emit_mode(EmitMode::Stdout);
                                config.set().edition(Edition::Edition2018);
                                let mut session = Session::new(config, Some(&mut buf));
                                session.format(Input::Text(code)).unwrap();
                            }

                            des.push_codeblock_safe(
                                &String::from_utf8(buf).unwrap()[7..],
                                Some("rs"),
                            );
                        }

                        des.push_codeblock_safe(res, None);
                    }
                    None => {
                        child.kill().unwrap();
                        child.wait().unwrap().code();

                        des.push("Timeout when running the thread");
                    }
                }

                des.push("Here's a image of your code ;)");

                m.content(des.build());

                let data = argon2::hash_raw(
                    msg.content.as_bytes(),
                    b"code_art",
                    &Config {
                        ad: &[],
                        hash_length: 192,
                        lanes: 1,
                        mem_cost: 256,
                        secret: &[],
                        time_cost: 1,
                        ..Default::default()
                    },
                )
                .unwrap();

                let img = RgbImage::from_raw(8, 8, data).unwrap();

                let img = imageops::resize(&img, 512, 512, FilterType::Nearest);

                let mut buf = Vec::new();

                png::PNGEncoder::new(&mut buf)
                    .encode(&img.into_raw(), 512, 512, ColorType::Rgb8)
                    .unwrap();

                m.add_file(AttachmentType::Bytes {
                    data: Cow::Owned(buf),
                    filename: String::from("code-art.png"),
                });

                m
            }) {
                error!("Error sending message: {:?}", why);
            }
        }
    }

    fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

fn main() -> Result<()> {
    init();

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let mut client = Client::new(&token, Handler)?;

    client.start()
}
