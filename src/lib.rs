extern crate curl;
use curl::easy::Easy;

extern crate html5ever;

mod html;
mod opengraph;

use html::HTML;

use std::str;

pub struct TCP {
    pub ip: String,
}

pub struct HTTP {
    pub content_type: String,
    pub headers: Vec<String>,
    pub url: String,
    pub body: String,
}

pub struct Webpage {
    pub tcp: TCP,
    pub http: HTTP,
    pub html: Option<HTML>,
}

pub fn fetch(url : &str) -> Webpage {
    let mut handle = Easy::new();

    // configure
    handle.timeout(std::time::Duration::from_secs(10)).unwrap();
    handle.follow_location(true).unwrap();
    handle.max_redirections(5).unwrap();
    handle.useragent("Webpage - rust crate").unwrap();

    handle.url(url).unwrap();

    let mut headers = Vec::new();
    let mut body = Vec::new();
    {
        let mut transfer = handle.transfer();
        transfer.header_function(|new_data| {
            headers.push(String::from_utf8_lossy(new_data).into_owned());
            true
        }).unwrap();

        transfer.write_function(|new_data| {
            body.extend_from_slice(new_data);
            Ok(new_data.len())
        }).unwrap();

        transfer.perform().unwrap();
    }
    let body = String::from_utf8_lossy(&body).into_owned();

    let tcp = TCP {
        ip: handle.primary_ip().unwrap().unwrap().to_string(),
    };
    let effective_url = handle.effective_url().unwrap().unwrap().to_string();
    let html = HTML::from_string(body.clone(), &effective_url);
    let http = HTTP {
        content_type: handle.content_type().unwrap().unwrap().to_string(),
        headers,
        url: effective_url,
        body: body,
    };

    Webpage {
        tcp, http, html
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
