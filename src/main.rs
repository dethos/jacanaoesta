use clap::{Arg, ArgAction, Command};
use rpassword;
use std::env;
use std::process;

fn main() {
    let matches = Command::new("jacanaoesta")
        .version("0.1.0")
        .author("Gonçalo Valério <gon@ovalerio.net>")
        .about("Find people that are no longer active in your Mastodon follow list.")
        .arg(Arg::new("instance").required(true))
        .arg(
            Arg::new("api-key")
                .short('k')
                .long("api-key")
                .help("Ask for API key")
                .action(ArgAction::SetTrue),
        )
        .get_matches();

    let ask_for_key = matches.get_flag("api-key");
    let api_key = match get_api_key(ask_for_key) {
        Ok(key) => key,
        _ => {
            println!("Could not find a valid API Key");
            process::exit(1);
        }
    };

    // TODO
}

fn get_api_key(ask: bool) -> Result<String, ()> {
    if ask {
        match rpassword::prompt_password("Paste API Key here:") {
            Ok(key) => return Ok(key),
            _ => Err(()),
        }
    } else {
        match env::var("JCNE_MAST_API_KEY") {
            Ok(key) => return Ok(key),
            _ => Err(()),
        }
    }
}
