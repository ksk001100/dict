use async_std::task;
use seahorse::{App, Context, Flag, FlagType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::process::exit;
use surf::http::Url;

#[derive(Serialize, Deserialize)]
struct Params<'a> {
    format: &'a str,
    action: &'a str,
    prop: &'a str,
    exintro: bool,
    explaintext: bool,
    redirects: &'a str,
    titles: &'a str,
}

#[derive(Deserialize)]
struct PageValue {
    extract: String,
}

#[derive(Deserialize)]
struct Query {
    pages: HashMap<String, PageValue>,
}

#[derive(Deserialize)]
struct Response {
    query: Query,
}

async fn search<'a>(lang: &'a str, text: &'a str) -> surf::Result<()> {
    let url = Url::parse(&format!("https://{}.wikipedia.org/w/api.php", lang))?;

    let params = Params {
        format: "json",
        action: "query",
        prop: "extracts",
        exintro: true,
        explaintext: true,
        redirects: "1",
        titles: &text,
    };

    let res = if let Ok(res) = surf::get(url).query(&params)?.recv_string().await {
        res
    } else {
        eprintln!("That language does not exist");
        exit(1);
    };

    let res: Response = match serde_json::from_str(&res) {
        Ok(res) => res,
        Err(_) => {
            eprintln!("Not found...");
            exit(1);
        }
    };

    match res.query.pages.values().next() {
        Some(page) => println!("{}", page.extract),
        None => eprintln!("Not found..."),
    }

    Ok(())
}

fn action(c: &Context) {
    task::block_on(async {
        let text = if let Some(text) = c.args.get(0) {
            text
        } else {
            c.help();
            exit(1);
        };

        let lang = match c.string_flag("lang") {
            Ok(lang) => lang,
            Err(_) => match env::var("LANG") {
                Ok(ref lang) => lang[..2].to_string(),
                Err(_) => "en".to_string(),
            },
        };

        search(&lang, &text).await.unwrap();
    });
}

fn main() {
    App::new(env!("CARGO_PKG_NAME"))
        .description(env!("CARGO_PKG_DESCRIPTION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .version(env!("CARGO_PKG_VERSION"))
        .usage(format!("{} [text]", env!("CARGO_PKG_NAME")))
        .action(action)
        .flag(
            Flag::new("lang", FlagType::String)
                .alias("l")
                .description("Language designation"),
        )
        .run(env::args().collect())
}
