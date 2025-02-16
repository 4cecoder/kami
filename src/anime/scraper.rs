use base64::{decode, encode};
use isahc::config::Configurable;
use isahc::{ReadResponseExt, Request, RequestExt};
use regex::Regex;

pub fn get_anime_html(url: &str) -> String {
    let req = Request::builder()
        .uri(url)
        .redirect_policy(isahc::config::RedirectPolicy::Follow)
        .header(
            "user-agent",
            "Mozilla/5.0 (X11; Linux x86_64; rv:99.0) Gecko/20100101 Firefox/100.0",
        )
        .body(())
        .unwrap();
    req.send().unwrap().text().unwrap()
}

pub fn get_ep_location(url: &str) -> String {
    let request = Request::builder()
        .method("HEAD")
        .uri(url)
        .header(
            "user-agent",
            "Mozilla/5.0 (X11; Linux x86_64; rv:99.0) Gecko/20100101 Firefox/100.0",
        )
        .body(())
        .unwrap();
    let response = request.send().unwrap();
    let headers = response.headers();
    let location = headers.get("location").unwrap();
    location.to_str().unwrap().to_string()
}

pub fn anime_names(query: String) -> Vec<String> {
    let url = format!("https://gogoanime.dk//search.html?keyword={}", query);
    //relpace all spaces with %20
    let url = url.replace(' ', "%20");
    let html = get_anime_html(&url);
    let re = Regex::new(r#"(?m)/category/([^"]*)"#).unwrap();
    let mut anime_list: Vec<String> = Vec::new();
    for cap in re.captures_iter(&html) {
        anime_list.push(cap.get(1).unwrap().as_str().trim().to_string());
    }
    anime_list.dedup();

    anime_list
}

pub fn get_anime_info(title: &str) -> (i32, u16) {
    let url = format!("https://animixplay.to/v1/{}", title);
    let html = get_anime_html(&url);
    let re = Regex::new(r#"(?m)var malid = '([0-9]*)'"#).unwrap();
    let mal_id = re
        .captures_iter(&html)
        .next()
        .unwrap()
        .get(1)
        .unwrap()
        .as_str()
        .trim()
        .to_string();

    let re = Regex::new(r#"(?m)"eptotal":([\d]+)"#).unwrap();
    let episodes = re
        .captures_iter(&html)
        .next()
        .unwrap()
        .get(1)
        .unwrap()
        .as_str()
        .trim()
        .to_string();
    (
        mal_id.parse::<i32>().unwrap_or(0),
        episodes.parse::<u16>().unwrap_or(0),
    )
}

pub fn anime_link(title: &str, ep: u64, provider: &str) -> (String, String) {
    let url = format!("https://animixplay.to/v1/{}", title);
    let html = get_anime_html(&url);
    let re = Regex::new(r#"(?m)\?id=([^&]+)"#).unwrap();
    let id1 = match re.captures_iter(&html).nth(ep as usize - 1) {
        Some(cap) => cap.get(1).unwrap().as_str().trim().to_string(),
        None => "".to_string(),
    };
    if id1 != "" {
        if provider == "vrv" {
            let title = format!("{} Episode {}", title.replace('-', " "), ep);
            let encoded_id1 = encode(&id1);
            let encoded_id2 = encode(&encoded_id1);
            let mut last_byte = encoded_id1.as_bytes()[encoded_id1.len() - 2];
            last_byte += 1;
            let mut new_encoded_id1 = encoded_id1.as_bytes().to_vec();
            new_encoded_id1.pop();
            new_encoded_id1.pop();
            new_encoded_id1.push(last_byte);
            let new_encoded_id1 = String::from_utf8(new_encoded_id1).unwrap();
            let ani_id = format!("cW9{}MVFhzM0dyVTh3ZTlP{}", new_encoded_id1, encoded_id2);
            let result = get_ep_location(format!("https://animixplay.to/api/{}", ani_id).as_str());
            //split the result into at # and return the second part
            let result: String = std::str::from_utf8(
                decode(result.split('#').nth(1).unwrap())
                    .unwrap()
                    .as_slice(),
            )
            .unwrap()
            .to_string();
            return (result, title);
        } else if provider == "gogo" {
            let title = format!("{} Episode {}", title.replace('-', " "), ep);
            let encoded_id1 = encode(&id1);
            let anime_id = encode(format!("{}LTXs3GrU8we9O{}", id1, encoded_id1));
            let html = format!("https://animixplay.to/api/cW9{}", anime_id);
            let url = get_ep_location(&html);
            let url = url.split('#').nth(1).unwrap();
            let url = std::str::from_utf8(&decode(url).unwrap())
                .unwrap()
                .to_string();
            return (url, title);
        } else {
            panic!("Invalid provider");
        }
    } else {
        let re = Regex::new(r#"(?m)r\.html#(.*)""#).unwrap();
        let id2 = re
            .captures_iter(&html)
            .next()
            .unwrap()
            .get(1)
            .unwrap()
            .as_str()
            .trim()
            .to_string();
        let url = decode(id2).unwrap();
        let url = std::str::from_utf8(&url).unwrap().to_string();
        let title = format!("{} Episode {}", title.replace('-', " "), ep);
        return (url, title);
    }
}
