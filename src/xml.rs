//! XML ユーティリティモジュール
//!
//! xml crate (xml-rs) を使った XML パース・生成の薄いラッパーを提供する。
//! 自前の文字列操作による XML 処理を置き換え、エスケープ漏れ・パース不備を防ぐ。

use std::str::FromStr;

/// S3 API の namespace
pub(crate) const S3_NS: &str = "http://s3.amazonaws.com/doc/2006-03-01/";

// -------------------------------------------------------
// パース用関数
// -------------------------------------------------------

/// 指定タグのテキスト内容を取得する（最初に見つかったもの）
pub(crate) fn extract_element(xml_str: &str, tag: &str) -> Option<String> {
    use xml::reader::{EventReader, XmlEvent};

    let reader = EventReader::from_str(xml_str);
    let mut inside_target = false;
    let mut text = String::new();

    for event in reader {
        match event {
            Ok(XmlEvent::StartElement { name, .. }) if name.local_name == tag => {
                inside_target = true;
                text.clear();
            }
            Ok(XmlEvent::Characters(s)) if inside_target => {
                text.push_str(&s);
            }
            Ok(XmlEvent::EndElement { name }) if inside_target && name.local_name == tag => {
                return Some(text);
            }
            Ok(XmlEvent::EndElement { .. }) if inside_target => {
                // ネストされた要素の終了タグは無視する
            }
            Err(_) => return None,
            _ => {}
        }
    }

    None
}

/// 親タグ内の子孫要素テキストを保持する構造体
///
/// 直接の子要素 (depth 2) のテキストに加え、ネストされた孫要素 (depth 3 以降)
/// のテキストも `path` でアクセスできる。同名タグの複数出現にも対応する
/// (`get_all`)。
pub(crate) struct ChildElements {
    /// (タグへのパス, テキスト) のリスト。
    /// パスは ["Owner", "DisplayName"] のようなネスト構造を表す。
    children: Vec<(Vec<String>, String)>,
}

impl ChildElements {
    /// 直接の子タグでテキストを取得する (最初に見つかったもの)
    pub(crate) fn get(&self, tag: &str) -> Option<&str> {
        self.children
            .iter()
            .find(|(path, _)| path.len() == 1 && path[0] == tag)
            .map(|(_, value)| value.as_str())
    }

    /// 同名直接子タグの複数出現を全て取得する (`<ChecksumAlgorithm>` 等)
    pub(crate) fn get_all(&self, tag: &str) -> Vec<&str> {
        self.children
            .iter()
            .filter(|(path, _)| path.len() == 1 && path[0] == tag)
            .map(|(_, value)| value.as_str())
            .collect()
    }

    /// ネストされた孫要素のテキストを取得する
    /// 例: `get_nested(&["Owner", "DisplayName"])`
    pub(crate) fn get_nested(&self, path: &[&str]) -> Option<&str> {
        self.children
            .iter()
            .find(|(p, _)| p.len() == path.len() && p.iter().zip(path).all(|(a, b)| a == b))
            .map(|(_, value)| value.as_str())
    }

    /// 直接子タグが存在するかを返す (テキスト空でもネスト要素のみでも `true`)
    pub(crate) fn has(&self, tag: &str) -> bool {
        self.children
            .iter()
            .any(|(path, _)| !path.is_empty() && path[0] == tag)
    }

    /// タグ名でテキストを取得し、FromStr でパースする
    pub(crate) fn get_parsed<T: FromStr>(&self, tag: &str) -> Option<T> {
        self.get(tag).and_then(|v| v.parse().ok())
    }
}

/// 指定した親タグの各出現に対してクロージャを呼び出す
pub(crate) fn for_each_element<F>(xml_str: &str, parent_tag: &str, mut f: F)
where
    F: FnMut(&ChildElements),
{
    use xml::reader::{EventReader, XmlEvent};

    let reader = EventReader::from_str(xml_str);
    let mut inside_parent = false;
    let mut depth: u32 = 0;
    // 親要素配下の現在のタグスタック (depth 2 以降を記録)
    let mut path_stack: Vec<String> = Vec::new();
    let mut current_text = String::new();
    let mut children: Vec<(Vec<String>, String)> = Vec::new();

    for event in reader {
        match event {
            Ok(XmlEvent::StartElement { name, .. })
                if !inside_parent && name.local_name == parent_tag =>
            {
                inside_parent = true;
                depth = 1;
                path_stack.clear();
                children.clear();
            }
            Ok(XmlEvent::StartElement { name, .. }) if inside_parent => {
                depth += 1;
                // depth 2 以降の要素をパススタックに積む
                path_stack.push(name.local_name.clone());
                current_text.clear();
            }
            Ok(XmlEvent::Characters(s)) if inside_parent && !path_stack.is_empty() => {
                current_text.push_str(&s);
            }
            Ok(XmlEvent::EndElement { name }) if inside_parent => {
                if depth >= 2 {
                    // パススタックの末尾と一致する終了タグなら、その時点までの
                    // テキストをパスと共に記録する。テキストが空 (ネスト要素のみ)
                    // でも記録する (`has()` で存在判定するため)。
                    if let Some(top) = path_stack.last()
                        && name.local_name == *top
                    {
                        children.push((path_stack.clone(), current_text.clone()));
                        path_stack.pop();
                    }
                    current_text.clear();
                }
                depth -= 1;
                if depth == 0 {
                    inside_parent = false;
                    f(&ChildElements {
                        children: children.clone(),
                    });
                    children.clear();
                    path_stack.clear();
                }
            }
            Err(_) => return,
            _ => {}
        }
    }
}

/// XML のルート要素が `<Error>` かどうかを判定する
///
/// CompleteMultipartUpload / CopyObject の 200 OK レスポンスで
/// ボディにエラーが含まれているかを判定するために使用する。
/// 正常レスポンスに偶然 `<Code>` タグが含まれていても誤判定しない。
pub(crate) fn has_error_root(xml_str: &str) -> bool {
    use xml::reader::{EventReader, XmlEvent};

    let reader = EventReader::from_str(xml_str);
    for event in reader {
        match event {
            Ok(XmlEvent::StartElement { name, .. }) => {
                return name.local_name == "Error";
            }
            Ok(XmlEvent::StartDocument { .. })
            | Ok(XmlEvent::ProcessingInstruction { .. })
            | Ok(XmlEvent::Comment(_))
            | Ok(XmlEvent::Whitespace(_)) => continue,
            _ => return false,
        }
    }
    false
}

/// S3 エラー XML から Code と Message を取得する
pub(crate) fn parse_s3_error(body: &[u8]) -> Option<(String, String)> {
    let text = std::str::from_utf8(body).ok()?;
    let code = extract_element(text, "Code")?;
    let message = extract_element(text, "Message").unwrap_or_default();
    Some((code, message))
}

// -------------------------------------------------------
// 生成用構造体
// -------------------------------------------------------

/// XML 文字列を生成するラッパー
pub(crate) struct XmlWriter {
    writer: xml::writer::EventWriter<Vec<u8>>,
}

impl XmlWriter {
    pub(crate) fn new() -> Self {
        let config = xml::EmitterConfig::new()
            .write_document_declaration(false)
            .perform_indent(false);
        let writer = config.create_writer(Vec::new());
        Self { writer }
    }

    /// namespace 付きルート要素を開始する
    pub(crate) fn start_ns(&mut self, tag: &str, ns: &str) {
        let event = xml::writer::XmlEvent::start_element(tag).default_ns(ns);
        self.writer.write(event).expect("XML write failed");
    }

    /// 子要素を開始する
    pub(crate) fn start(&mut self, tag: &str) {
        let event = xml::writer::XmlEvent::start_element(tag);
        self.writer.write(event).expect("XML write failed");
    }

    /// テキストを書く（自動エスケープ）
    pub(crate) fn text(&mut self, content: &str) {
        let event = xml::writer::XmlEvent::characters(content);
        self.writer.write(event).expect("XML write failed");
    }

    /// 現在の要素を閉じる
    pub(crate) fn end(&mut self) {
        let event = xml::writer::XmlEvent::end_element();
        self.writer.write(event).expect("XML write failed");
    }

    /// <tag>text</tag> を一度に書く（リーフ要素用）
    pub(crate) fn element(&mut self, tag: &str, content: &str) {
        self.start(tag);
        self.text(content);
        self.end();
    }

    /// 完了して String を返す
    pub(crate) fn finish(self) -> String {
        let inner = self.writer.into_inner();
        String::from_utf8(inner).expect("XML output is not valid UTF-8")
    }
}
