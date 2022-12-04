#![allow(non_snake_case)]
#![allow(unused_variables)]

use clap::{value_parser, Arg, ArgAction, Command, ValueSource};
use http::{header::HeaderName, HeaderMap, HeaderValue};
use reqwest::{Client, ClientBuilder, Response};
use std::str::FromStr;
use text2art::{BasicFonts, Font, Printer};

#[tokio::main]
async fn main() {
    let LOGO: String = Printer::with_font(Font::from_basic(BasicFonts::Bell).unwrap())
        .render_text(format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")).as_str())
        .unwrap()
        .to_string();

    let matches = Command::new(env!("CARGO_PKG_NAME"))
        .before_help(LOGO.as_str())
        // .about("guilty")
        .help_template("{before-help}{about}\n{usage-heading} {usage}\n\n{all-args}")
        .version(env!("CARGO_PKG_VERSION"))
        .arg(
            Arg::new("url")
                .short('u')
                .long("url")
                .required(true)
                .takes_value(true)
                .help("Request URL"),
        )
        .arg(
            Arg::new("method")
                .short('m')
                .long("method")
                .takes_value(true)
                .action(clap::ArgAction::Set)
                .default_value("GET")
                .hide_default_value(true)
                .possible_values([
                    "GET", "HEAD", "POST", "PUT", "PATCH", "DELETE", "CONNECT", "OPTIONS", "TRACE",
                ])
                .ignore_case(true)
                .requires_if("post", "data")
                .hide_possible_values(true)
                .help("Request method"),
        )
        .arg(
            Arg::new("user_agent")
                .short('U')
                .long("user_agent")
                .takes_value(true)
                .default_value(ua_generator::ua::spoof_ua())
                .hide_default_value(true)
                .help("User agent"),
        )
        .arg(
            Arg::new("data")
                .short('d')
                .long("data")
                .takes_value(true)
                .default_value("{}")
                .hide_default_value(true)
                .help("Request data"),
        )
        .arg(
            Arg::new("silent")
                .short('s')
                .long("silent")
                .action(clap::ArgAction::SetTrue)
                .help("Don't print response body"),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(clap::ArgAction::SetTrue)
                .help("Make program more verbose"),
        )
        .arg(
            Arg::new("noprefix")
                .short('r')
                .long("noprefix")
                .action(clap::ArgAction::SetTrue)
                .help("Remove prefix from verbose/debug logs"),
        )
        .arg(
            Arg::new("crowns")
                .short('c')
                .long("crowns")
                .takes_value(true)
                .help("How many crowns to perform")
                .value_parser(value_parser!(i64))
                .hide_default_value(true)
                .default_value("1"),
        )
        .arg(
            Arg::new("timeout")
                .short('t')
                .long("timeout")
                .takes_value(true)
                .help("Timeout between requests in seconds")
                .value_parser(value_parser!(u64))
                .hide_default_value(true)
                .default_value("0"),
        )
        .arg(
            Arg::new("header")
                .action(ArgAction::Append)
                .short('H')
                .long("header")
                .takes_value(true)
                .help("Add header to request")
                .value_parser(value_parser!(String)),
        )
        .get_matches()
        .to_owned();
    let URL: &str = matches.get_one::<String>("url").unwrap();
    let METHOD: &str = matches.get_one::<String>("method").unwrap();
    let USER_AGENT: &str = matches.get_one::<String>("user_agent").unwrap();
    let DATA: &str = matches.get_one::<String>("data").unwrap();
    let CROWNS: &i64 = matches.get_one::<i64>("crowns").unwrap();
    let TIMEOUT: &u64 = matches.get_one::<u64>("timeout").unwrap();

    let VERBOSE: bool = matches.get_flag("verbose");
    let NOPREFIX: bool = matches.get_flag("noprefix");
    let SILENT: bool = matches.get_flag("silent");

    macro_rules! debug {
        ($string: expr) => {
            match NOPREFIX {
                true => match VERBOSE {
                    true => println!("{}", $string),
                    false => (),
                },

                false => match VERBOSE {
                    true => println!("[{}] {}", env!("CARGO_PKG_NAME"), $string),
                    false => (),
                },
            }
        };

        ($string: expr, $verbosity: expr) => {
            match NOPREFIX {
                true => match $verbosity {
                    true => println!("{}", $string),
                    false => (),
                },

                false => match $verbosity {
                    true => println!("[{}] {}", env!("CARGO_PKG_NAME"), $string),
                    false => (),
                },
            }
        };
    }

    let client_raw: ClientBuilder = ClientBuilder::new().user_agent(USER_AGENT);
    let mut client_headers: HeaderMap = HeaderMap::new();

    let mut headers: Vec<&str> = vec![];

    if let Some(values) = matches.get_many::<String>("header") {
        headers = values.map(|s| s.as_str()).collect::<Vec<_>>();
    }

    for header in headers {
        let headerSplit: (&str, &str) = header.split_once(": ").expect(
            "Header must follow 'Header: Value' format. (Include the whitespace after the colon)",
        );

        let headerName: &str = headerSplit.0;
        let headerValue: &str = headerSplit.1;

        debug!(format!("Header {:#?} : {:#?}", headerName, headerValue));

        client_headers.insert(
            HeaderName::from_str(headerName).unwrap(),
            HeaderValue::from_str(headerValue).unwrap(),
        );
    }

    debug!(format!("URL: {}", URL));
    debug!(format!("Method: {}", METHOD));

    if matches.value_source("user_agent").unwrap() == ValueSource::CommandLine {
        debug!(format!("User-Agent: {}", USER_AGENT));
    }

    if METHOD.to_uppercase() == "POST" {
        debug!(format!("Data: {}", DATA));
    }

    debug!(format!("GCrowns: {}", CROWNS));
    debug!(format!("Timeout: {}", TIMEOUT));

    debug!(format!("Verbose: {}", VERBOSE));
    debug!(format!("Silent: {}", SILENT));

    let total = std::time::Instant::now();
    // let sum: i64 = 0;

    let client: Client = client_raw
        .user_agent(USER_AGENT)
        .default_headers(client_headers)
        .build()
        .unwrap()
        .to_owned();

    for i in 1..*CROWNS + 1 {
        // let (result) = tokio::join!();
        // request(client.clone(), URL, METHOD, SILENT).await;

        let current = std::time::Instant::now();
        let req: Response;

        match std::string::String::from(METHOD).to_uppercase().as_str() {
            "POST" => req = client.post(URL).body(DATA.to_owned()).send().await.unwrap(),
            "PUT" => req = client.put(URL).send().await.unwrap(),
            "PATCH" => req = client.patch(URL).send().await.unwrap(),
            "DELETE" => req = client.delete(URL).send().await.unwrap(),
            "HEAD" => req = client.head(URL).send().await.unwrap(),
            _ => req = client.get(URL).send().await.unwrap(),
        }

        debug!(format!("Request {} with status code {}", i, req.status()));
        debug!(format!("{:#?} elapsed", current.elapsed()));
        debug!(format!("{}", req.text().await.unwrap()), !SILENT);

        if *CROWNS > 1 {
            tokio::time::sleep(std::time::Duration::from_secs(TIMEOUT.to_owned())).await;
        }
    }

    debug!(format!(
        "{} requests totaled to {:#?}",
        CROWNS,
        total.elapsed()
    ))
}
