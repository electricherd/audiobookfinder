///! Mainly pages to be delivered and triggered from webui.
///! There are some static pages and few dynamical pages.
use super::{
    super::{super::config, PeerRepresentation},
    WebServerState,
};
use actix_files as fs;
use actix_web::{
    http::StatusCode,
    web::{self, HttpResponse},
    Responder,
};
use std::{
    ffi::OsString,
    string::String,
    sync::{Arc, Mutex},
};

/// dynamic development files
//#[get("/app.js")]
//fn app_js() -> Result<fs::NamedFile> {
//Ok(fs::NamedFile::open("src/ctrl/webui/js/app.js")?)
//}

#[allow(dead_code)]
pub async fn dyn_devel_html() -> impl Responder {
    fs::NamedFile::open("src/ctrl/webui/html/single_page.html")
}

#[allow(dead_code)]
pub async fn dyn_devel_js() -> impl Responder {
    fs::NamedFile::open("src/ctrl/webui/js/app.js")
}

pub async fn single_page(state: web::Data<Arc<Mutex<WebServerState>>>) -> impl Responder {
    // change state
    let mut data = state.lock().unwrap();
    let id = data.id;
    *(data.nr_connections.lock().unwrap()) += 1;

    let id_page = replace_static_content(*config::webui::HTML_PAGE, &id);

    HttpResponse::build(StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body(id_page)
}

pub async fn bootstrap_css(path: web::Path<(String,)>) -> impl Responder {
    let css = &*path.0;
    let output = match css {
        "bootstrap.css" => Some(*config::webui::bootstrap::CSS),
        "bootstrap.css.map" => Some(*config::webui::bootstrap::CSS_MAP),
        "bootstrap.min.css" => Some(*config::webui::bootstrap::CSS_MIN),
        "bootstrap.min.css.map" => Some(*config::webui::bootstrap::CSS_MIN_MAP),
        "bootstrap-grid.css" => Some(*config::webui::bootstrap::CSS_GRID),
        "bootstrap-grid.css.map" => Some(*config::webui::bootstrap::CSS_GRID_MAP),
        "bootstrap-grid.min.css" => Some(*config::webui::bootstrap::CSS_GRID_MIN),
        "bootstrap-grid.min.css.map" => Some(*config::webui::bootstrap::CSS_GRID_MIN_MAP),
        "bootstrap-reboot.css" => Some(*config::webui::bootstrap::CSS_REBOOT),
        "bootstrap-reboot.css.map" => Some(*config::webui::bootstrap::CSS_REBOOT_MAP),
        "bootstrap-reboot.min.css" => Some(*config::webui::bootstrap::CSS_REBOOT_MIN),
        "bootstrap-reboot.min.css.map" => Some(*config::webui::bootstrap::CSS_REBOOT_MIN_MAP),
        _ => {
            error!("CSS: not found {}", css);
            None
        }
    };
    if let Some(content) = output {
        HttpResponse::build(StatusCode::OK)
            .content_type("text/css; charset=utf-8")
            .body(content)
    } else {
        HttpResponse::build(StatusCode::NOT_FOUND)
            .content_type("text/css; charset=utf-8")
            .body("")
    }
}

pub async fn bootstrap_js(path: web::Path<(String,)>) -> impl Responder {
    let js = &*path.0;
    let output = match js {
        "bootstrap.js" => Some(*config::webui::bootstrap::JS),
        "bootstrap.js.map" => Some(*config::webui::bootstrap::JS_MAP),
        "bootstrap.min.js" => Some(*config::webui::bootstrap::JS_MIN),
        "bootstrap.min.js.map" => Some(*config::webui::bootstrap::JS_MIN_MAP),
        "bootstrap.bundle.js" => Some(*config::webui::bootstrap::JS_BUNDLE),
        "bootstrap.bundle.js.map" => Some(*config::webui::bootstrap::JS_BUNDLE_MAP),
        "bootstrap.bundle.min.js" => Some(*config::webui::bootstrap::JS_BUNDLE_MIN),
        "bootstrap.bundle.min.js.map" => Some(*config::webui::bootstrap::JS_BUNDLE_MIN_MAP),
        _ => {
            error!("JS: not found {}", js);
            None
        }
    };
    if let Some(content) = output {
        HttpResponse::build(StatusCode::OK)
            .content_type("application/javascript; charset=utf-8")
            .body(content)
    } else {
        HttpResponse::build(StatusCode::NOT_FOUND)
            .content_type("application/javascript; charset=utf-8")
            .body("")
    }
}

pub async fn js_app(state: web::Data<Arc<Mutex<WebServerState>>>) -> impl Responder {
    let data = state.lock().unwrap();
    let id = data.id;

    let output = replace_static_content(*config::webui::JS_APP, &id);
    HttpResponse::build(StatusCode::OK)
        .content_type("application/javascript; charset=utf-8")
        .body(output)
}

fn replace_static_content(html_in: &str, id: &PeerRepresentation) -> String {
    // short inline struct
    struct ReplaceStatic<'a> {
        r: &'a str,
        c: String,
    }

    let id_string = std::format!("{:x?}", id);
    let hostname = hostname::get()
        .unwrap_or(OsString::from("undefined"))
        .into_string()
        .unwrap_or(String::from("undefined"));

    // todo: getting to many replacements, should be done more efficiently!!! Like a regex or so
    //       or function searching for "<!--" and then a LUT!
    let changers: [ReplaceStatic; 7] = [
        ReplaceStatic {
            r: config::net::HTML_REPLACE_STATIC_URL_SOURCE,
            c: config::net::HTML_URL_SOURCE.to_string(),
        },
        ReplaceStatic {
            r: config::net::HTML_REPLACE_STATIC_WEBSOCKET_ADDR,
            c: config::net::WEBSOCKET_ADDR.to_string(),
        },
        ReplaceStatic {
            r: config::net::HTML_REPLACE_STATIC_PORT_WEBSOCKET,
            c: config::net::PORT_WEBSOCKET.to_string(),
        },
        ReplaceStatic {
            r: config::webui::HTML_REPLACE_WEBSOCKET,
            c: config::net::WEBSOCKET_ADDR.to_string(),
        },
        ReplaceStatic {
            r: config::webui::HTML_REPLACE_PEER_SHA2,
            c: id_string,
        },
        ReplaceStatic {
            r: config::webui::HTML_REPLACE_HOSTNAME,
            c: hostname,
        },
        ReplaceStatic {
            r: config::webui::HTML_REPLACE_PEER_PAGE,
            c: config::webui::PEER_PAGE.to_string(),
        },
    ];
    let mut replace_this = html_in.to_string();

    for replacer in &changers {
        replace_this = str::replace(&replace_this, &replacer.r, &replacer.c);
    }
    replace_this
}
