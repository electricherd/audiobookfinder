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
    fs::NamedFile::open("src/ctrl/webui/html/main_page.html")
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
    let port = data.web_port;

    let id_page = replace_static_content(*config::webui::HTML_PAGE, &id, port);

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
    let port = data.web_port;

    let output = replace_static_content(*config::webui::JS_APP, &id, port);
    HttpResponse::build(StatusCode::OK)
        .content_type("application/javascript; charset=utf-8")
        .body(output)
}

fn replace_static_content(html_in: &str, id: &PeerRepresentation, port: u16) -> String {
    // short inline struct

    let id_string = std::format!("{:x?}", id);
    let hostname = hostname::get()
        .unwrap_or(OsString::from("undefined"))
        .into_string()
        .unwrap_or(String::from("undefined"));

    let changers: [ReplaceStatic; 7] = [
        ReplaceStatic {
            r: config::net::HTML_REPLACE_STATIC_URL_SOURCE,
            c: config::net::HTML_URL_SOURCE.to_string(),
        },
        ReplaceStatic {
            r: config::net::HTML_REPLACE_STATIC_WEB_ADDR,
            c: config::net::WEB_ADDR.to_string(),
        },
        ReplaceStatic {
            r: config::net::HTML_REPLACE_STATIC_WEB_PORT,
            c: port.to_string(),
        },
        ReplaceStatic {
            r: config::webui::HTML_REPLACE_WEBSOCKET,
            c: config::net::WEB_ADDR.to_string(),
        },
        ReplaceStatic {
            r: config::webui::HTML_REPLACE_PEER_HASH,
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
    linear_LUT_replacer(html_in, &changers)
}

struct ReplaceStatic<'a> {
    r: &'a str,
    c: String,
}

#[allow(non_snake_case)]
fn linear_LUT_replacer(replace_this: &str, changers: &[ReplaceStatic]) -> String {
    let left_bracket = config::webui::HTML_REPLACER_BEGIN;
    let right_bracket = config::webui::HTML_REPLACER_END;

    let mut replace_this = replace_this.to_string();

    let mut begin = 0;
    while let Some(found_pattern_begin) = replace_this[begin..].find(left_bracket) {
        match &replace_this[found_pattern_begin..].find(right_bracket) {
            None => break,
            Some(pattern_end) => {
                let mrange = std::ops::Range {
                    start: begin + found_pattern_begin + left_bracket.len(),
                    end: found_pattern_begin + pattern_end,
                };
                let part = &replace_this[mrange];
                if let Some(good) = changers.iter().find(|el| el.r == part) {
                    let erange = std::ops::Range {
                        start: begin + found_pattern_begin,
                        end: found_pattern_begin + pattern_end + right_bracket.len(),
                    };
                    replace_this.replace_range(erange, &good.c);
                    begin += &good.c.len() + found_pattern_begin;
                } else {
                    begin += pattern_end;
                }
            }
        }
    }
    replace_this
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(non_snake_case)]
    fn quick_replace_LUT_tests() {
        let changers: [ReplaceStatic; 5] = [
            ReplaceStatic {
                r: config::net::HTML_REPLACE_STATIC_URL_SOURCE,
                c: config::net::HTML_URL_SOURCE.to_string(),
            },
            ReplaceStatic {
                r: "CAT",
                c: "cat".to_string(),
            },
            ReplaceStatic {
                r: "JUMP",
                c: "jumped".to_string(),
            },
            ReplaceStatic {
                r: "ON",
                c: " on ".to_string(),
            },
            ReplaceStatic {
                r: "TABLE",
                c: "table".to_string(),
            },
        ];
        let replace_this = "The <!---CAT---> <!---JUMP---><!---ON--->the <!---TABLE--->.";
        let return_value = linear_LUT_replacer(replace_this, &changers);
        let expect = "The cat jumped on the table.";
        assert_eq!(return_value, expect);

        let replace_this = "<!---JUMP---> <!---CAT---> <!---ON---> <!---CAT--->.";
        let return_value = linear_LUT_replacer(replace_this, &changers);
        let expect = "jumped cat  on  cat.";
        assert_eq!(return_value, expect);

        let replace_this = "<!---JUMP---><!---CAT--->";
        let return_value = linear_LUT_replacer(replace_this, &changers);
        let expect = "jumpedcat";
        assert_eq!(return_value, expect);

        let replace_this = "<!--JUMP--><!---CAT---><!--- -->";
        let return_value = linear_LUT_replacer(replace_this, &changers);
        let expect = "<!--JUMP-->cat<!--- -->";
        assert_eq!(return_value, expect);
    }
}
