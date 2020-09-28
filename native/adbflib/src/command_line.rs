//! Command line modules: has one function which takes input parameters from commandline
//! and parses them.
use adbfbinlib::common::config;

static APP_TITLE: &str = concat!("The audiobook finder (", env!("CARGO_PKG_NAME"), ")");

static ARG_NET: &str = "net";
static ARG_TUI: &str = "tui";
static ARG_WEBUI: &str = "webui";
static ARG_KEEP_ALIVE: &str = "keep";
static ARG_BROWSER: &str = "browser";
static ARG_BROWSER_PORT: &str = "port";

static INPUT_FOLDERS: &str = "folders";

const AUTHORS: &'static str = env!("CARGO_PKG_AUTHORS");
const DESCRIPTION: &'static str = env!("CARGO_PKG_DESCRIPTION");

/// Get all start values which are passed from command line
///
/// Get all start values and returns the following tuple
/// ui_paths,
/// has_tui,
/// has_webui,
/// has_net,
/// keep_alive,
/// open_browser,
/// web_port,
/// has_ui,
pub fn get_start_values() -> (Vec<String>, bool, bool, bool, bool, bool, u16, bool) {
    let parse_args = clap::App::new(APP_TITLE)
        .version(config::net::VERSION)
        .author(AUTHORS)
        .about(DESCRIPTION)
        .long_about::<&str>(
            &[
                &DESCRIPTION,
                "\n\
                 It reads data from possibly multiple given path(s). Via local network it searches \
                 for other instances of the program, and will later exchange data securely.\n\
                 All information gathered will be used to find duplicates, versions of \
                 different quality, different tags for same content (spelling, \
                 incompleteness).\n\
                 For documentation see: ",
                &config::net::HOMEPAGE,
                "\n \
                 A program to learn, embrace, and love Rust! \n\
                 Have fun!",
            ]
            .concat(),
        )
        .arg(
            clap::Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Sets custom config file (not implemented yet)")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name(ARG_TUI)
                .short("t")
                .long(ARG_TUI)
                .help("Run with TUI")
                .takes_value(false),
        )
        .arg(
            clap::Arg::with_name(ARG_WEBUI)
                .short("w")
                .long(ARG_WEBUI)
                .help("Run with-in a webui.")
                .takes_value(false),
        )
        .arg(
            clap::Arg::with_name(ARG_BROWSER_PORT)
                .short("p")
                .long(ARG_BROWSER_PORT)
                .help(
                    &vec![
                        "Define port for webui (only works with webui).\nDefault port is:"
                            .to_string(),
                        config::net::WEB_PORT_DEFAULT.to_string(),
                        " but please choose from 8080 to 8099)".to_string(),
                    ]
                    .join(""),
                )
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name(ARG_NET)
                .short("n")
                .long(ARG_NET)
                .help("With net search for other audiobookfinders running in local network.")
                .takes_value(false),
        )
        .arg(
            clap::Arg::with_name(ARG_KEEP_ALIVE)
                .short("k")
                .long(ARG_KEEP_ALIVE)
                .help(
                    "With keep alive process will continue even after search has been performed.\
                     This should be used and will be turned on automatically with net, webui\
                     browser option.",
                )
                .takes_value(false),
        )
        .arg(
            clap::Arg::with_name(ARG_BROWSER)
                .short("b")
                .long(ARG_BROWSER)
                .help("Shall browser not be openend automatically (only works with webui).")
                .takes_value(false),
        )
        .arg(
            clap::Arg::with_name(INPUT_FOLDERS)
                .help(
                    &[
                        "Sets multiple input folder(s) to be searched for audio files. (Max ",
                        &config::data::PATHS_MAX.to_string(),
                        " input folders will be used!)",
                    ]
                    .concat(),
                )
                .multiple(true)
                .required(false),
        )
        .get_matches();
    // tricky thing, but I really like that
    let all_pathes = if let Some(correct_input) = parse_args.values_of(INPUT_FOLDERS) {
        correct_input.collect()
    } else {
        vec!["."]
    };

    //
    // check argments if tui and net search is needed
    //
    let has_arg = |x: &str| parse_args.is_present(x);

    let has_tui = has_arg(ARG_TUI);
    let has_webui = has_arg(ARG_WEBUI);
    let has_net = has_arg(ARG_NET);
    let has_port = has_arg(ARG_BROWSER_PORT);
    let mut keep_alive = has_arg(ARG_KEEP_ALIVE);
    let open_browser = !has_arg(ARG_BROWSER);

    //
    // section for better user experience
    // todo: think it over
    if has_webui || has_net {
        keep_alive = true;
    }
    // not mutable
    let keep_alive = keep_alive;

    let web_port = {
        let web_default_string = config::net::WEB_PORT_DEFAULT.to_string();
        // when to write to console
        let has_to_write_console = !has_tui && has_webui;

        let parsed_value = parse_args
            .value_of(ARG_BROWSER_PORT)
            .unwrap_or_else(|| {
                if has_to_write_console && has_port {
                    println!(
                        "Port argument was bad, using default port {}!",
                        config::net::WEB_ADDR
                    );
                }
                &web_default_string
            })
            .parse::<u16>()
            .unwrap_or_else(|_| {
                if has_to_write_console && has_port {
                    println!(
                        "Invalid port input, using default port {}!",
                        &web_default_string
                    );
                }
                config::net::WEB_PORT_DEFAULT
            });
        if parsed_value < config::net::WEB_PORT_MIN || parsed_value > config::net::WEB_PORT_MAX {
            if has_to_write_console {
                let web_max_string = config::net::WEB_PORT_MAX.to_string();
                let web_min_string = config::net::WEB_PORT_MIN.to_string();
                println!(
                    "Port not in range {} .. {}, using default {}!",
                    &web_min_string, &web_max_string, &web_default_string
                );
            }
            config::net::WEB_PORT_DEFAULT
        } else {
            parsed_value
        }
    };

    // extended help for certain option combinations
    if !has_tui && has_webui && !open_browser {
        println!(
            "Open http://{}:{} to start!",
            config::net::WEB_ADDR,
            web_port
        );
        println!("The webui needs to get the start signal from there");
    }

    // either one will have a ui, representing data and error messages
    let has_ui = has_tui || has_webui;

    // 1) convert to strings
    let unchecked_strings = all_pathes.iter().map(|s| s.to_string()).collect();
    (
        unchecked_strings,
        has_tui,
        has_webui,
        has_net,
        keep_alive,
        open_browser,
        web_port,
        has_ui,
    )
}
