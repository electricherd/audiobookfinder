///! Mainly pages to be delivered and triggered from webui.
///! There are some static pages and few dynamical pages.
use super::{
    super::super::{common::config, net::subs::peer_representation::PeerRepresentation},
    WebServerState,
};
use actix_web::{
    http::StatusCode,
    web::{self, HttpResponse},
    Responder,
};
use regex::{Captures, Regex, Replacer};
use std::{
    ffi::OsString,
    string::String,
    sync::{Arc, Mutex},
};

pub async fn single_page(state: web::Data<Arc<Mutex<WebServerState>>>) -> impl Responder {
    // change state
    let data = state.lock().unwrap();
    let id = data.id;
    *(data.nr_connections.lock().unwrap()) += 1;
    let port = data.web_port;

    let id_page = replace_static_content(*config::webui::HTML_PAGE, &id, port);

    HttpResponse::build(StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body(id_page)
}

pub async fn bootstrap_css(path: web::Path<String>) -> impl Responder {
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

pub async fn bootstrap_js(path: web::Path<String>) -> impl Responder {
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

    lazy_static! {
        static ref LICENSES: String = config::webui::LICENSES
            .iter()
            .enumerate()
            .map(|(index, txt)| {
                if index < config::webui::LICENSE_NR - 1 {
                    [txt, "\n---------------------\n"].join("").to_string()
                } else {
                    txt.to_string()
                }
            })
            .collect();
    }

    let changers: [ReplaceStatic; 10] = [
        ReplaceStatic {
            r: config::webui::HTML_REPLACE_STATIC_URL_SOURCE,
            c: config::net::HOMEPAGE.to_string(),
        },
        ReplaceStatic {
            r: config::webui::HTML_REPLACE_STATIC_WEB_ADDR,
            c: config::net::WEB_ADDR.to_string(),
        },
        ReplaceStatic {
            r: config::webui::HTML_REPLACE_STATIC_WEB_PORT,
            c: port.to_string(),
        },
        ReplaceStatic {
            r: config::webui::HTML_REPLACE_WEB_SOCKET,
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
        ReplaceStatic {
            r: config::webui::HTML_REPLACE_PATHS_MAX,
            c: config::data::PATHS_MAX.to_string(),
        },
        ReplaceStatic {
            r: config::webui::HTML_REPLACE_VERSION,
            c: config::net::VERSION.to_string(),
        },
        ReplaceStatic {
            r: config::webui::HTML_REPLACE_STATIC_LICENSE,
            c: LICENSES.to_string(),
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
    lazy_static! {
        static ref RE_WINDOWS: Regex = Regex::new(
            &[
                config::webui::HTML_REPLACER_BEGIN,
                "(?P<txt>([[:alnum:]]|_)+)",
                config::webui::HTML_REPLACER_END
            ]
            .concat()
        )
        .unwrap();
    }
    let rep = HTMLReplacer::new(changers);
    RE_WINDOWS.replace_all(replace_this, rep).to_string()
}

struct HTMLReplacer<'a> {
    changers: &'a [ReplaceStatic<'a>],
}

impl<'a> HTMLReplacer<'a> {
    fn new(changers: &'a [ReplaceStatic<'a>]) -> Self {
        Self {
            changers: &changers,
        }
    }
}

impl<'a> Replacer for HTMLReplacer<'a> {
    fn replace_append(&mut self, caps: &Captures, dst: &mut String) {
        let matched = caps.name("txt").unwrap();
        dst.push_str(
            &self
                .changers
                .iter()
                .find(|r| r.r == matched.as_str())
                .and_then(|s| Some(s.c.clone()))
                .or_else(|| {
                    warn!("This HTML-replacer is not defined '{}'", matched.as_str());
                    Some("".to_string())
                })
                .unwrap(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(non_snake_case)]
    fn quick_replace_LUT_tests() {
        let changers: [ReplaceStatic; 6] = [
            ReplaceStatic {
                r: config::webui::HTML_REPLACE_STATIC_URL_SOURCE,
                c: config::net::HOMEPAGE.to_string(),
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
            ReplaceStatic {
                r: config::webui::HTML_REPLACE_VERSION,
                c: config::net::VERSION.to_string(),
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

        let replace_this = [
            "<!--JUMP--><!---",
            &config::webui::HTML_REPLACE_VERSION,
            "---><!--- -->",
        ]
        .concat();
        let return_value = linear_LUT_replacer(&replace_this, &changers);
        let expect = ["<!--JUMP-->", &config::net::VERSION, "<!--- -->"].concat();
        assert_eq!(return_value, expect);
    }
}
