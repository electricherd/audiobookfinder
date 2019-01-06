//! This module just holds as a collection of all modules configuration.
//! Some configuration should be hold in its module but more general parts
//! (which later may be configurable by a config file) should be put here.

/// The network related configurations
#[allow(non_camel_case_types)]
pub mod net {
    use std::net::{IpAddr, Ipv4Addr}; // later SocketAddr

    /// MDNS service address space
    pub static MDNS_SERVICE_NAME: &str = "_http._tcp"; // "_tcp.local"

    pub static MDNS_REGISTER_NAME: &str = "adbf";
    pub static MDNS_TIMEOUT_SEC: u16 = 5;

    pub static SSH_CLIENT_USERNAME: &str = "e";

    pub static PORT_MDNS: u16 = 80;
    pub static PORT_SSH: u16 = 8080;
    pub static PORT_WEBSOCKET: u16 = 8088;

    pub static HOST: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

    // later pub static WEBSOCKET_ADDR: SocketAddr = SocketAddr::new(HOST, PORT_WEBSOCKET);
    pub static WEBSOCKET_ADDR: &str = "localhost:8088";

    // key can be generated by
    // ssh-keygen -f ~/.adbf/client_key.priv -N 'adbf' -t ed25519

    /// private key path, relatively to env_home
    pub static SSH_CLIENT_SEC_KEY_PATH: &str = "/.adbf/client_key.priv";
    pub static SSH_CLIENT_SEC_KEY_PASSWD: &str = "adbf";

    pub struct changeable {}
}

/// The TUI related configurations of a more general purpose
pub mod tui {
    pub static ALIVE_REFRESH: u64 = 80;
}

/// The webui related configurations
pub mod webui {
    // 3rd party jquery hard-wired, and needed by bootstrap
    pub mod jquery {
        lazy_static! {
            pub static ref JS_JQUERY: &'static str =
                include_str!("ctrl/webui/3rdparty/jquery-3.3.1/jquery.min.js");
        }
    }
    // 3rd party bootstrap hard-wired: css 317kb + js 107kb + fonts 216kb
    pub mod bootstrap {
        lazy_static! {
            pub static ref JS: &'static str =
                include_str!("ctrl/webui/3rdparty/bootstrap-3.3.7/js/bootstrap.js");
            pub static ref JS_MIN: &'static str =
                include_str!("ctrl/webui/3rdparty/bootstrap-3.3.7/js/bootstrap.min.js");
            pub static ref CSS: &'static str =
                include_str!("ctrl/webui/3rdparty/bootstrap-3.3.7/css/bootstrap.css");
            pub static ref CSS_MIN: &'static str =
                include_str!("ctrl/webui/3rdparty/bootstrap-3.3.7/css/bootstrap.min.css");
            pub static ref CSS_THEME: &'static str =
                include_str!("ctrl/webui/3rdparty/bootstrap-3.3.7/css/bootstrap-theme.css");
            pub static ref CSS_THEME_MIN: &'static str =
                include_str!("ctrl/webui/3rdparty/bootstrap-3.3.7/css/bootstrap-theme.min.css");
            pub static ref CSS_MAP: &'static str =
                include_str!("ctrl/webui/3rdparty/bootstrap-3.3.7/css/bootstrap.css.map");
            pub static ref CSS_MIN_MAP: &'static str =
                include_str!("ctrl/webui/3rdparty/bootstrap-3.3.7/css/bootstrap.min.css.map");
            pub static ref CSS_THEME_MAP: &'static str =
                include_str!("ctrl/webui/3rdparty/bootstrap-3.3.7/css/bootstrap-theme.css.map");
            pub static ref CSS_THEME_MIN_MAP: &'static str =
                include_str!("ctrl/webui/3rdparty/bootstrap-3.3.7/css/bootstrap-theme.min.css.map");
            pub static ref FONT_EOT: &'static [u8] = include_bytes!(
                "ctrl/webui/3rdparty/bootstrap-3.3.7/fonts/glyphicons-halflings-regular.eot"
            );
            pub static ref FONT_TTF: &'static [u8] = include_bytes!(
                "ctrl/webui/3rdparty/bootstrap-3.3.7/fonts/glyphicons-halflings-regular.ttf"
            );
            pub static ref FONT_WOFF: &'static [u8] = include_bytes!(
                "ctrl/webui/3rdparty/bootstrap-3.3.7/fonts/glyphicons-halflings-regular.woff"
            );
            pub static ref FONT_WOFF2: &'static [u8] = include_bytes!(
                "ctrl/webui/3rdparty/bootstrap-3.3.7/fonts/glyphicons-halflings-regular.woff2"
            );
            pub static ref FONT_SVG: &'static [u8] = include_bytes!(
                "ctrl/webui/3rdparty/bootstrap-3.3.7/fonts/glyphicons-halflings-regular.svg"
            );
        }
    }
    // own pages
    lazy_static! {
        pub static ref HTML_PAGE: &'static str = include_str!("ctrl/webui/html/single_page.html");
        pub static ref JS_APP: &'static str = include_str!("ctrl/webui/js/app.js");
        pub static ref FAVICON: &'static [u8] = include_bytes!("ctrl/webui/gfx/favicon.png");
        pub static ref PIC_SHEEP: &'static str = include_str!("ctrl/webui/gfx/sheep.svg");
    }
    pub static HTML_REPLACE_UUID: &str = "<!--UUID-->";
    pub static HTML_REPLACE_WEBSOCKET: &str = "<!--WEBSOCKET-->";
}

/// The data related configurations of a more general purpose
pub mod data {
    /// ignore those tree_magic extensions. like m3u
    pub static IGNORE_AUDIO_FORMATS: [&str; 1] = ["x-mpegurl"];
}
