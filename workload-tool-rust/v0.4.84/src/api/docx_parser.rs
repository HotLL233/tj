use quick_xml::Reader;
use quick_xml::events::Event;
use std::io::BufReader;

pub struct DocxResult { pub html: String, pub toc: Vec<TocItem> }
#[derive(Clone, serde::Serialize)]
pub struct TocItem { pub id: String, pub text: String, pub level: u8, pub children: Vec<TocItem> }

pub fn parse_docx(data: &[u8]) -> Result<DocxResult, String> {
    let reader = std::io::Cursor::new(data);
    let mut archive = zip::ZipArchive::new(reader).map_err(|e| format!("无法打开docx: {}", e))?;

    let mut heading_styles: Vec<String> = Vec::new();
    if let Ok(mut sf) = archive.by_name("word/styles.xml") {
        let mut buf = Vec::new();
        std::io::copy(&mut sf, &mut buf).map_err(|e| format!("读styles: {}", e))?;
        let mut r = Reader::from_reader(BufReader::new(&buf[..]));
        let mut buf2 = Vec::new();
        let mut in_style = false;
        let mut style_id = String::new();
        let mut style_name = String::new();
        loop {
            match r.read_event_into(&mut buf2) {
                Ok(Event::Start(e)) => {
                    let name = e.name();
                    let tag = String::from_utf8_lossy(name.as_ref());
                    if tag == "w:style" {
                        in_style = true;
                        style_id = String::from_utf8_lossy(&e.attributes().filter_map(|a| a.ok()).find(|a| a.key.as_ref() == b"w:styleId").map(|a| a.value.to_vec()).unwrap_or_default()).to_string();
                    }
                    if in_style && tag == "w:name" {
                        style_name = String::from_utf8_lossy(&e.attributes().filter_map(|a| a.ok()).find(|a| a.key.as_ref() == b"w:val").map(|a| a.value.to_vec()).unwrap_or_default()).to_string();
                    }
                }
                Ok(Event::End(e)) => {
                    if String::from_utf8_lossy(e.name().as_ref()) == "w:style" {
                        if style_name.to_lowercase().starts_with("heading") || style_name.contains("标题") { heading_styles.push(style_id.clone()); }
                        in_style = false;
                    }
                }
                Ok(Event::Eof) => break,
                _ => {}
            }
        }
    }

    let mut df = archive.by_name("word/document.xml").map_err(|e| format!("找不到document.xml: {}", e))?;
    let mut doc_buf = Vec::new();
    std::io::copy(&mut df, &mut doc_buf).map_err(|e| format!("读document: {}", e))?;

    let mut r = Reader::from_reader(BufReader::new(&doc_buf[..]));
    let mut buf = Vec::new();
    let mut html = String::from("<div>");
    let mut toc: Vec<TocItem> = Vec::new();
    let mut para_text = String::new();
    let mut in_para = false;
    let mut current_style: Option<String> = None;
    let mut heading_level: Option<u8> = None;
    let mut runs_text = String::new();
    let mut in_run = false;
    let mut run_bold = false;

    loop {
        match r.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_lowercase();
                match tag.as_str() {
                    "w:p" => { in_para = true; para_text.clear(); current_style = None; heading_level = None; }
                    "w:pstyle" => {
                        current_style = Some(String::from_utf8_lossy(&e.attributes().filter_map(|a| a.ok()).find(|a| a.key.as_ref() == b"w:val").map(|a| a.value.to_vec()).unwrap_or_default()).to_string());
                        if let Some(ref sid) = current_style {
                            for hs in &heading_styles {
                                if sid == hs {
                                    let num = sid.chars().filter(|c| c.is_ascii_digit()).collect::<String>().parse::<u8>().unwrap_or(1);
                                    heading_level = Some(num); break;
                                }
                            }
                        }
                    }
                    "w:r" => { in_run = true; runs_text.clear(); run_bold = false; }
                    "w:b" => { run_bold = true; }
                    _ => {}
                }
            }
            Ok(Event::End(e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_lowercase();
                match tag.as_str() {
                    "w:p" => {
                        if !para_text.is_empty() {
                            if let Some(lv) = heading_level {
                                let id = format!("h_{}", toc.len() + 1);
                                html.push_str(&format!("<h{} id=\"{}\">{}</h{}>", lv, id, para_text, lv));
                                toc.push(TocItem { id, text: para_text.clone(), level: lv, children: vec![] });
                            } else {
                                html.push_str(&format!("<p>{}</p>", para_text));
                            }
                        }
                        in_para = false;
                    }
                    "w:r" => {
                        in_run = false;
                        if run_bold { para_text.push_str(&format!("<b>{}</b>", runs_text)); }
                        else { para_text.push_str(&runs_text); }
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(t)) => {
                if in_run { runs_text.push_str(&t.unescape().unwrap_or_default()); }
            }
            Ok(Event::Eof) => break,
            _ => {}
        }
    }

    html.push_str("</div>");
    Ok(DocxResult { html, toc })
}
