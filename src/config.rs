//! This module just holds as a collection of all modules configuration.
//! Some configuration should be hold in its module but more general parts
//! (which later may be configurable by a config file) should be put here.

/// The network related configurations
#[allow(non_camel_case_types)]
pub mod net {
    pub static WEB_ADDR: &str = "localhost";
    pub static WEB_PORT_DEFAULT: u16 = 8088;
    pub static WEB_PORT_MAX: u16 = 8099;
    pub static WEB_PORT_MIN: u16 = 8080;

    pub static HTML_REPLACE_STATIC_WEB_ADDR: &str = "WEBSOCKET_ADDR";
    pub static HTML_REPLACE_STATIC_WEB_PORT: &str = "PORT_WEBSOCKET";
    pub static HTML_REPLACE_STATIC_URL_SOURCE: &str = "URL_SOURCE";

    pub static HTML_URL_SOURCE: &str = "https://github.com/electricherd/audiobookfinder";
}

/// The TUI related configurations of a more general purpose
pub mod tui {
    pub static ALIVE_REFRESH_MSEC: u64 = 200;
}

/// The webui related configurations
pub mod webui {
    // 3rd party jquery hard-wired, and needed by bootstrap
    pub mod jquery {
        lazy_static! {
            pub static ref JS_JQUERY: &'static str =
                include_str!("ctrl/webui/3rdparty/jquery/jquery-3.5.1.min.js");
        }
    }
    // 3rd party bootstrap hard-wired: css 317kb + js 107kb + fonts 216kb
    pub mod bootstrap {
        lazy_static! {
            //
            // javascript
            //
            // bootstrap
            pub static ref JS: &'static str =
                include_str!("ctrl/webui/3rdparty/bootstrap-4.5.0/js/bootstrap.js");
            pub static ref JS_MAP: &'static str =
                include_str!("ctrl/webui/3rdparty/bootstrap-4.5.0/js/bootstrap.js.map");
            // bootstrap min
            pub static ref JS_MIN: &'static str =
                include_str!("ctrl/webui/3rdparty/bootstrap-4.5.0/js/bootstrap.min.js");
            pub static ref JS_MIN_MAP: &'static str =
                include_str!("ctrl/webui/3rdparty/bootstrap-4.5.0/js/bootstrap.min.js.map");
            // bootstrap bundle
            pub static ref JS_BUNDLE: &'static str =
                include_str!("ctrl/webui/3rdparty/bootstrap-4.5.0/js/bootstrap.bundle.js");
            pub static ref JS_BUNDLE_MAP: &'static str =
                include_str!("ctrl/webui/3rdparty/bootstrap-4.5.0/js/bootstrap.bundle.js.map");
            // bootstrap bundle min
            pub static ref JS_BUNDLE_MIN: &'static str =
                include_str!("ctrl/webui/3rdparty/bootstrap-4.5.0/js/bootstrap.bundle.min.js");
            pub static ref JS_BUNDLE_MIN_MAP: &'static str =
                include_str!("ctrl/webui/3rdparty/bootstrap-4.5.0/js/bootstrap.bundle.min.js.map");
            //
            // css
            //
            // bootstrap
            pub static ref CSS: &'static str =
                include_str!("ctrl/webui/3rdparty/bootstrap-4.5.0/css/bootstrap.css");
            pub static ref CSS_MAP: &'static str =
                include_str!("ctrl/webui/3rdparty/bootstrap-4.5.0/css/bootstrap.css.map");
            // boostrap min
            pub static ref CSS_MIN: &'static str =
                include_str!("ctrl/webui/3rdparty/bootstrap-4.5.0/css/bootstrap.min.css");
            pub static ref CSS_MIN_MAP: &'static str =
                include_str!("ctrl/webui/3rdparty/bootstrap-4.5.0/css/bootstrap.min.css.map");
            // bootstrap grid
            pub static ref CSS_GRID: &'static str =
                include_str!("ctrl/webui/3rdparty/bootstrap-4.5.0/css/bootstrap-grid.css");
            pub static ref CSS_GRID_MAP: &'static str =
                include_str!("ctrl/webui/3rdparty/bootstrap-4.5.0/css/bootstrap-grid.css.map");
            // bootstrap grid min
            pub static ref CSS_GRID_MIN: &'static str =
                include_str!("ctrl/webui/3rdparty/bootstrap-4.5.0/css/bootstrap-grid.min.css");
            pub static ref CSS_GRID_MIN_MAP: &'static str =
                include_str!("ctrl/webui/3rdparty/bootstrap-4.5.0/css/bootstrap-grid.min.css.map");
            // bootstrap reboot
            pub static ref CSS_REBOOT: &'static str =
                include_str!("ctrl/webui/3rdparty/bootstrap-4.5.0/css/bootstrap-reboot.css");
            pub static ref CSS_REBOOT_MAP: &'static str =
                include_str!("ctrl/webui/3rdparty/bootstrap-4.5.0/css/bootstrap-reboot.css.map");
            // bootstrap reboot min
            pub static ref CSS_REBOOT_MIN: &'static str =
                include_str!("ctrl/webui/3rdparty/bootstrap-4.5.0/css/bootstrap-reboot.min.css");
            pub static ref CSS_REBOOT_MIN_MAP: &'static str =
                include_str!("ctrl/webui/3rdparty/bootstrap-4.5.0/css/bootstrap-reboot.min.css.map");
        }
    }
    // own pages
    lazy_static! {
        pub static ref HTML_PAGE: &'static str = include_str!("ctrl/webui/html/main_page.html");
        pub static ref PEER_PAGE: &'static str = include_str!("ctrl/webui/html/peer_page.html");
        pub static ref JS_APP: &'static str = include_str!("ctrl/webui/js/app.js");
        pub static ref JS_WS_EVENT_DISPATCHER: &'static str =
            include_str!("ctrl/webui/js/ws_events_dispatcher.js");
        pub static ref FAVICON: &'static [u8] = include_bytes!("ctrl/webui/gfx/favicon.png");
        pub static ref PIC_SHEEP: &'static str = include_str!("ctrl/webui/gfx/sheep.svg");
    }
    pub static HTML_REPLACER_BEGIN: &str = "<!---";
    pub static HTML_REPLACER_END: &str = "--->";
    pub static HTML_REPLACE_PEER_HASH: &str = "PEER_HASH";
    pub static HTML_REPLACE_HOSTNAME: &str = "HOSTNAME";
    pub static HTML_REPLACE_WEBSOCKET: &str = "WEBSOCKET";
    pub static HTML_REPLACE_PEER_PAGE: &str = "PEER_PAGE";
    pub static HTML_REPLACE_PATHS_MAX: &str = "PATHS_MAX";
}

/// The data related configurations of a more general purpose
pub mod data {
    /// ignore those tree_magic extensions. like m3u
    pub static IGNORE_AUDIO_FORMATS: [&str; 1] = ["x-mpegurl"];
    /// max of paths to be used by program todo: use it everywhere it needs to be
    pub static PATHS_MAX: usize = 10;
}
