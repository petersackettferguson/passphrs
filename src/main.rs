use anyhow::Result;
use clipboard::{ClipboardProvider, ClipboardContext};
use clap::Parser;
use rand::prelude::*;
use thiserror::Error;

const DEFAULT_LIST: &str = "eff_large_wordlist.txt";

#[derive(Clone, Debug, Parser)]
#[clap(author, version, about, long_about=None)]
#[clap(about = "Generate a passphrase.")]
struct Cli {
    /// Shows debugging information with varying levels of detail
    #[clap(short, long, parse(from_occurrences))]
    debug: usize,

    /// Displays a sample passphrase along with information about its security
    #[clap(short, long, parse(from_flag))]
    info: bool,

    /// Duration to wait before clearing clipboard
    #[clap(default_value_t = 5, short, long, parse(try_from_str))]
    wait: u64,

    /// Sets passphrase length
    #[clap(default_value_t = 7, short, long, parse(try_from_str))]
    length: usize,

    /// Sets separator between words
    #[clap(default_value = " ", short, long)]
    separator: String,

    /// Set salt length
    #[clap(default_value_t = 1, long = "sl", parse(try_from_str))]
    salt_length: usize,

    /// Set valid salt characters
    #[clap(default_value = "0123456789", long = "sc")]
    salt_chars: String,

    /// Set word case. 0: lowercase, 1: capitalized, 2: uppercase
    #[clap(default_value_t = 1, short, long, parse(try_from_str))]
    case: usize,

    /// Use a custom word list at the given location
    #[clap(short, long, value_name="FILE")]
    path: Option<String>,
}

#[derive(Debug, Error)]
pub enum PassphraseError {
    #[error("Initialization error: {0}")]
    InitializationError(String),
    #[error("Generation error: {0}")]
    GenerationError(String),
    #[error("Output error: {0}")]
    OutputError(String),
}

fn get_list(path: Option<&String>)
    -> Result<Vec<String>> {
    let raw: String = if let Some(path_) = path {
        println!("Reading word list from {}...", path_);
        std::fs::read_to_string(path_)?
    } else {
        std::fs::read_to_string(DEFAULT_LIST)?
    };

    let list = raw.lines();
    Ok(list.map(|w| w
                .trim_matches(|c: char| !c.is_alphabetic())
                .to_ascii_lowercase())
       .filter(|w| w.len() > 1)
       .collect())
}

fn build_passphrase(
    list: &Vec<String>, 
    length: usize, 
    separator: &str, 
    salt_length: usize,
    salt_chars: &str,
    case: usize,
) -> String {
    let mut rng = rand::prelude::thread_rng();
    let salt_pos = rng.gen_range(0..length);
    let mut phrase = String::new();
    for i in 0..length {
        if i == 0 {
            let mut word = list[rng.gen_range(0..list.len())].clone();
            match case {
                0 => word.make_ascii_lowercase(),
                1 => { word.get_mut(0..1).unwrap().make_ascii_uppercase()},
                2 => word.make_ascii_uppercase(),
                _ => {}
            };

            phrase += word.as_str();
        } else {
            let mut word = list[rng.gen_range(0..list.len())].clone();
            match case {
                0 => word.make_ascii_lowercase(),
                1 => { word.get_mut(0..1).unwrap().make_ascii_uppercase()},
                2 => word.make_ascii_uppercase(),
                _ => {}
            };

            phrase += separator;
            phrase += word.as_str();
        }

        if i == salt_pos {
            for _ in 0..salt_length {
                phrase.push(salt_chars.chars().nth(rng.gen_range(0..salt_chars.len())).unwrap())
            }
        }
    };

    phrase
}

fn entropy(
    list_len: usize,
    phrase_len: usize,
    salt_len: usize,
    salt_chars: &String,
) -> (f64, f64) {
    // N is the total number of valid combinations
    let mut c: f64 = (list_len as f64).powi(phrase_len as i32);
    if salt_len > 0 {
        c *= (phrase_len * (salt_chars.len().pow(salt_len as u32))) as f64;
    }

    let entropy = (c as f64).log2();
    (entropy, entropy / 7.0)
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let wait = cli.wait.clone();
    let length = cli.length.clone();
    let separator = cli.separator.clone();
    let salt_length = cli.salt_length.clone();
    let salt_chars = cli.salt_chars.clone();
    let case = cli.case.clone();

    if cli.debug > 0 { eprintln!("{:?}", cli.clone()) };

    let word_list_result = get_list(
        cli.path.as_ref());

    let word_list = word_list_result?;

    if cli.debug > 1 {
        for i in 0..3 {
            eprintln!("{}", word_list[i])
        }
    }

    if cli.info {
        let sample_phrase = build_passphrase(
            &word_list, 
            length, 
            &separator, 
            salt_length, 
            &salt_chars, 
            case);

        println!("DO NOT USE THIS PASSPHRASE. Most shells log their history in an unencrypted file. Instead run this program in the standard mode to copy a passphrase directly to your clipboard.");
        println!();
        println!("Sample: {}", sample_phrase);
        let (entropy, equivalent) = entropy(word_list.len(), length, cli.salt_length, &salt_chars);
        println!("Entropy: {:.2}", entropy);
        println!("This is equivalent to a {:.2}-character password of random ASCII characters", equivalent);
    } else {
        let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
        if let Err(err_) = ctx.set_contents(build_passphrase(
            &word_list, 
            length, 
            &separator, 
            salt_length, 
            &salt_chars, 
            case)) {
            eprintln!("Could not set clipboard contents: {}", err_);
        };

        if wait != 0 {
            std::thread::sleep(std::time::Duration::from_secs(wait));
            if let Err(err_) = ctx.set_contents(String::new()) {
                eprintln!("Could not clear clipboard contents: {}", err_);
            }
        }
    }

    Ok(())
}

