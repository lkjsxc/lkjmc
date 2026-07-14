use crate::web::html::escape;

pub struct WebReply {
    pub status: u16,
    pub content_type: &'static str,
    pub headers: Vec<(&'static str, String)>,
    pub body: String,
}

pub(crate) fn page(title: &str, body: String, csrf: Option<&str>) -> WebReply {
    let logout = csrf.map(|value| format!("<form method=post action=/web/logout><input type=hidden name=_csrf value=\"{}\"><button>logout</button></form>", escape(value))).unwrap_or_default();
    reply(200, "text/html; charset=utf-8", &format!("<!doctype html><title>{}</title><link rel=stylesheet href=/web/static/style.css><main>{logout}{body}</main>", escape(title)))
}

pub(crate) fn reply(status: u16, content_type: &'static str, body: &str) -> WebReply {
    WebReply {
        status,
        content_type,
        headers: Vec::new(),
        body: body.to_string(),
    }
}
