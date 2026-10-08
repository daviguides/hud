//! The JSON document of structured output: a [`Node`] written in the canonical form of
//! `structured-json.md` and read back from it. Pure: strings in, strings out.

use crate::model::{Direction, Justify, Node, NodeError, NodeKind, TaskState};

/// A JSON value, only as far as the document `hud/1` needs: no floating point, no negatives.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Json {
    Null,
    Bool(bool),
    Int(u64),
    Str(String),
    Arr(Vec<Json>),
    Obj(Vec<(String, Json)>),
}

// ---------------------------------------------------------------------------
// Node to JSON
// ---------------------------------------------------------------------------

fn string(text: &str) -> Json {
    Json::Str(text.to_string())
}

fn optional(text: &Option<String>) -> Json {
    text.as_deref().map_or(Json::Null, string)
}

fn object(pairs: Vec<(&str, Json)>) -> Json {
    Json::Obj(pairs.into_iter().map(|(k, v)| (k.to_string(), v)).collect())
}

fn strings(items: &[String]) -> Json {
    Json::Arr(items.iter().map(|item| string(item)).collect())
}

fn nodes(items: &[Node]) -> Json {
    Json::Arr(items.iter().map(to_json_value).collect())
}

fn justify_name(justify: Justify) -> &'static str {
    match justify {
        Justify::Default | Justify::Left => "left",
        Justify::Center => "center",
        Justify::Right => "right",
        Justify::Full => "full",
    }
}

fn direction_name(direction: Direction) -> &'static str {
    match direction {
        Direction::None => "none",
        Direction::Row => "row",
        Direction::Column => "column",
    }
}

fn to_json_value(node: &Node) -> Json {
    let kind = |name: &str| ("type", string(name));
    match &node.0 {
        NodeKind::Text(text) => object(vec![kind("text"), ("text", string(text))]),
        NodeKind::Table {
            title,
            caption,
            columns,
            rows,
        } => object(vec![
            kind("table"),
            ("title", optional(title)),
            ("caption", optional(caption)),
            (
                "columns",
                Json::Arr(
                    columns
                        .iter()
                        .map(|(header, justify)| {
                            object(vec![
                                ("header", string(header)),
                                ("justify", string(justify_name(*justify))),
                            ])
                        })
                        .collect(),
                ),
            ),
            (
                "rows",
                Json::Arr(rows.iter().map(|row| strings(row)).collect()),
            ),
        ]),
        NodeKind::Panel {
            title,
            subtitle,
            body,
        } => object(vec![
            kind("panel"),
            ("title", optional(title)),
            ("subtitle", optional(subtitle)),
            ("body", to_json_value(body)),
        ]),
        NodeKind::Tree { label, children } => object(vec![
            kind("tree"),
            ("label", string(label)),
            ("children", nodes(children)),
        ]),
        NodeKind::Progress(tasks) => object(vec![
            kind("progress"),
            (
                "tasks",
                Json::Arr(
                    tasks
                        .iter()
                        .map(|task| {
                            object(vec![
                                ("description", string(&task.description)),
                                ("completed", Json::Int(task.completed)),
                                ("total", Json::Int(task.total)),
                                ("finished", Json::Bool(task.finished)),
                                ("visible", Json::Bool(task.visible)),
                            ])
                        })
                        .collect(),
                ),
            ),
        ]),
        NodeKind::Error {
            message,
            causes,
            hint,
        } => object(vec![
            kind("error"),
            ("message", string(message)),
            ("causes", strings(causes)),
            ("hint", optional(hint)),
        ]),
        NodeKind::Columns { title, items } => object(vec![
            kind("columns"),
            ("title", optional(title)),
            ("items", nodes(items)),
        ]),
        NodeKind::Layout {
            name,
            ratio,
            size,
            visible,
            direction,
            content,
            children,
        } => object(vec![
            kind("layout"),
            ("name", optional(name)),
            ("ratio", Json::Int(*ratio)),
            ("size", size.map_or(Json::Null, Json::Int)),
            ("visible", Json::Bool(*visible)),
            ("direction", string(direction_name(*direction))),
            (
                "content",
                content.as_deref().map_or(Json::Null, to_json_value),
            ),
            ("children", nodes(children)),
        ]),
        NodeKind::Group(items) => object(vec![kind("group"), ("items", nodes(items))]),
        NodeKind::Padding {
            top,
            right,
            bottom,
            left,
            content,
        } => object(vec![
            kind("padding"),
            ("top", Json::Int(*top)),
            ("right", Json::Int(*right)),
            ("bottom", Json::Int(*bottom)),
            ("left", Json::Int(*left)),
            ("content", to_json_value(content)),
        ]),
    }
}

// ---------------------------------------------------------------------------
// The canonical writer: Python's json.dumps(indent=2, ensure_ascii=False) plus a newline
// ---------------------------------------------------------------------------

fn indent(out: &mut String, depth: usize) {
    for _ in 0..depth {
        out.push_str("  ");
    }
}

fn quote(out: &mut String, text: &str) {
    out.push('"');
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
}

fn write(out: &mut String, json: &Json, depth: usize) {
    match json {
        Json::Null => out.push_str("null"),
        Json::Bool(true) => out.push_str("true"),
        Json::Bool(false) => out.push_str("false"),
        Json::Int(n) => out.push_str(&n.to_string()),
        Json::Str(text) => quote(out, text),
        Json::Arr(items) if items.is_empty() => out.push_str("[]"),
        Json::Arr(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push('\n');
                indent(out, depth + 1);
                write(out, item, depth + 1);
            }
            out.push('\n');
            indent(out, depth);
            out.push(']');
        }
        Json::Obj(pairs) if pairs.is_empty() => out.push_str("{}"),
        Json::Obj(pairs) => {
            out.push('{');
            for (i, (key, value)) in pairs.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push('\n');
                indent(out, depth + 1);
                quote(out, key);
                out.push_str(": ");
                write(out, value, depth + 1);
            }
            out.push('\n');
            indent(out, depth);
            out.push('}');
        }
    }
}

// ---------------------------------------------------------------------------
// JSON text to a JSON value
// ---------------------------------------------------------------------------

const MAX_DEPTH: usize = 128;

struct Parser<'a> {
    bytes: &'a [u8],
    pos: usize,
}

type Parsed<T> = Result<T, NodeError>;

impl<'a> Parser<'a> {
    fn fail<T>(&self, what: &str) -> Parsed<T> {
        Err(NodeError::new(format!("{what} at byte {}", self.pos)))
    }

    fn skip_space(&mut self) {
        while matches!(self.bytes.get(self.pos), Some(b' ' | b'\n' | b'\r' | b'\t')) {
            self.pos += 1;
        }
    }

    fn eat(&mut self, byte: u8) -> bool {
        if self.bytes.get(self.pos) == Some(&byte) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn expect(&mut self, byte: u8) -> Parsed<()> {
        if self.eat(byte) {
            Ok(())
        } else {
            self.fail(&format!("expected `{}`", byte as char))
        }
    }

    fn literal(&mut self, word: &str, value: Json) -> Parsed<Json> {
        if self.bytes[self.pos..].starts_with(word.as_bytes()) {
            self.pos += word.len();
            Ok(value)
        } else {
            self.fail("unknown literal")
        }
    }

    fn value(&mut self, depth: usize) -> Parsed<Json> {
        if depth > MAX_DEPTH {
            return self.fail("nesting too deep");
        }
        self.skip_space();
        match self.bytes.get(self.pos) {
            None => self.fail("unexpected end"),
            Some(b'{') => self.object(depth),
            Some(b'[') => self.array(depth),
            Some(b'"') => Ok(Json::Str(self.string()?)),
            Some(b't') => self.literal("true", Json::Bool(true)),
            Some(b'f') => self.literal("false", Json::Bool(false)),
            Some(b'n') => self.literal("null", Json::Null),
            Some(b'0'..=b'9') => self.number(),
            Some(b'-') => self.fail("numbers must be non-negative integers"),
            Some(_) => self.fail("unexpected character"),
        }
    }

    fn number(&mut self) -> Parsed<Json> {
        let start = self.pos;
        while matches!(self.bytes.get(self.pos), Some(b'0'..=b'9')) {
            self.pos += 1;
        }
        if matches!(self.bytes.get(self.pos), Some(b'.' | b'e' | b'E')) {
            return self.fail("numbers must be non-negative integers");
        }
        let digits = &self.bytes[start..self.pos];
        if digits.len() > 1 && digits[0] == b'0' {
            return self.fail("leading zero");
        }
        match std::str::from_utf8(digits)
            .ok()
            .and_then(|text| text.parse::<u64>().ok())
        {
            Some(n) => Ok(Json::Int(n)),
            None => self.fail("integer out of range"),
        }
    }

    fn hex4(&mut self) -> Parsed<u32> {
        let end = self.pos + 4;
        let Some(slice) = self.bytes.get(self.pos..end) else {
            return self.fail("short \\u escape");
        };
        let mut value = 0;
        for &byte in slice {
            let digit = match byte {
                b'0'..=b'9' => byte - b'0',
                b'a'..=b'f' => byte - b'a' + 10,
                b'A'..=b'F' => byte - b'A' + 10,
                _ => return self.fail("bad \\u escape"),
            };
            value = value * 16 + u32::from(digit);
        }
        self.pos = end;
        Ok(value)
    }

    fn string(&mut self) -> Parsed<String> {
        self.expect(b'"')?;
        let mut out = String::new();
        loop {
            let Some(&byte) = self.bytes.get(self.pos) else {
                return self.fail("unterminated string");
            };
            match byte {
                b'"' => {
                    self.pos += 1;
                    return Ok(out);
                }
                b'\\' => {
                    self.pos += 1;
                    let Some(&escape) = self.bytes.get(self.pos) else {
                        return self.fail("unterminated escape");
                    };
                    self.pos += 1;
                    match escape {
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        b'b' => out.push('\u{8}'),
                        b'f' => out.push('\u{c}'),
                        b'n' => out.push('\n'),
                        b'r' => out.push('\r'),
                        b't' => out.push('\t'),
                        b'u' => {
                            let first = self.hex4()?;
                            let code = if (0xD800..0xDC00).contains(&first) {
                                if !(self.eat(b'\\') && self.eat(b'u')) {
                                    return self.fail("unpaired surrogate");
                                }
                                let second = self.hex4()?;
                                if !(0xDC00..0xE000).contains(&second) {
                                    return self.fail("unpaired surrogate");
                                }
                                0x10000 + ((first - 0xD800) << 10) + (second - 0xDC00)
                            } else {
                                first
                            };
                            match char::from_u32(code) {
                                Some(c) => out.push(c),
                                None => return self.fail("unpaired surrogate"),
                            }
                        }
                        _ => return self.fail("bad escape"),
                    }
                }
                0..=0x1f => return self.fail("control character in a string"),
                _ => {
                    let rest = &self.bytes[self.pos..];
                    let width = match byte {
                        0..=0x7f => 1,
                        0xc0..=0xdf => 2,
                        0xe0..=0xef => 3,
                        _ => 4,
                    };
                    let Some(chunk) = rest.get(..width).and_then(|b| std::str::from_utf8(b).ok())
                    else {
                        return self.fail("invalid UTF-8");
                    };
                    out.push_str(chunk);
                    self.pos += width;
                }
            }
        }
    }

    fn array(&mut self, depth: usize) -> Parsed<Json> {
        self.expect(b'[')?;
        let mut items = Vec::new();
        self.skip_space();
        if self.eat(b']') {
            return Ok(Json::Arr(items));
        }
        loop {
            items.push(self.value(depth + 1)?);
            self.skip_space();
            if self.eat(b',') {
                continue;
            }
            self.expect(b']')?;
            return Ok(Json::Arr(items));
        }
    }

    fn object(&mut self, depth: usize) -> Parsed<Json> {
        self.expect(b'{')?;
        let mut pairs: Vec<(String, Json)> = Vec::new();
        self.skip_space();
        if self.eat(b'}') {
            return Ok(Json::Obj(pairs));
        }
        loop {
            self.skip_space();
            let key = self.string()?;
            if pairs.iter().any(|(k, _)| *k == key) {
                return self.fail("duplicate key");
            }
            self.skip_space();
            self.expect(b':')?;
            pairs.push((key, self.value(depth + 1)?));
            self.skip_space();
            if self.eat(b',') {
                continue;
            }
            self.expect(b'}')?;
            return Ok(Json::Obj(pairs));
        }
    }
}

fn parse(text: &str) -> Parsed<Json> {
    let mut parser = Parser {
        bytes: text.as_bytes(),
        pos: 0,
    };
    let value = parser.value(0)?;
    parser.skip_space();
    if parser.pos != parser.bytes.len() {
        return parser.fail("text after the document");
    }
    Ok(value)
}

// ---------------------------------------------------------------------------
// JSON value to Node
// ---------------------------------------------------------------------------

fn bad<T>(message: impl Into<String>) -> Result<T, NodeError> {
    Err(NodeError::new(message))
}

/// The fields of one object, checked against the exact set of keys of its type.
struct Fields<'a>(&'a [(String, Json)]);

impl<'a> Fields<'a> {
    fn of(json: &'a Json, what: &str, keys: &[&str]) -> Result<Fields<'a>, NodeError> {
        let Json::Obj(pairs) = json else {
            return bad(format!("{what} must be an object"));
        };
        if pairs.len() != keys.len() || !keys.iter().all(|k| pairs.iter().any(|(p, _)| p == k)) {
            return bad(format!("{what} must have exactly the keys {keys:?}"));
        }
        Ok(Fields(pairs))
    }

    fn get(&self, key: &str) -> &'a Json {
        self.0
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v)
            .unwrap_or(&Json::Null)
    }

    fn string(&self, key: &str) -> Result<String, NodeError> {
        match self.get(key) {
            Json::Str(s) => Ok(s.clone()),
            _ => bad(format!("`{key}` must be a string")),
        }
    }

    fn optional(&self, key: &str) -> Result<Option<String>, NodeError> {
        match self.get(key) {
            Json::Null => Ok(None),
            Json::Str(s) => Ok(Some(s.clone())),
            _ => bad(format!("`{key}` must be a string or null")),
        }
    }

    fn int(&self, key: &str) -> Result<u64, NodeError> {
        match self.get(key) {
            Json::Int(n) => Ok(*n),
            _ => bad(format!("`{key}` must be a non-negative integer")),
        }
    }

    fn optional_int(&self, key: &str) -> Result<Option<u64>, NodeError> {
        match self.get(key) {
            Json::Null => Ok(None),
            Json::Int(n) => Ok(Some(*n)),
            _ => bad(format!("`{key}` must be a non-negative integer or null")),
        }
    }

    fn boolean(&self, key: &str) -> Result<bool, NodeError> {
        match self.get(key) {
            Json::Bool(b) => Ok(*b),
            _ => bad(format!("`{key}` must be true or false")),
        }
    }

    fn array(&self, key: &str) -> Result<&'a [Json], NodeError> {
        match self.get(key) {
            Json::Arr(items) => Ok(items),
            _ => bad(format!("`{key}` must be an array")),
        }
    }

    fn strings(&self, key: &str) -> Result<Vec<String>, NodeError> {
        self.array(key)?
            .iter()
            .map(|item| match item {
                Json::Str(s) => Ok(s.clone()),
                _ => bad(format!("`{key}` must hold strings")),
            })
            .collect()
    }
}

fn type_of(json: &Json) -> Result<&str, NodeError> {
    match json {
        Json::Obj(pairs) => match pairs.iter().find(|(k, _)| k == "type") {
            Some((_, Json::Str(name))) => Ok(name),
            _ => bad("a node needs a string `type`"),
        },
        _ => bad("a node must be an object"),
    }
}

fn justify_of(name: &str) -> Result<Justify, NodeError> {
    Ok(match name {
        "left" => Justify::Left,
        "center" => Justify::Center,
        "right" => Justify::Right,
        "full" => Justify::Full,
        other => return bad(format!("unknown justify `{other}`")),
    })
}

fn direction_of(name: &str) -> Result<Direction, NodeError> {
    Ok(match name {
        "none" => Direction::None,
        "row" => Direction::Row,
        "column" => Direction::Column,
        other => return bad(format!("unknown direction `{other}`")),
    })
}

fn node_list(items: &[Json], only: Option<&str>) -> Result<Vec<Node>, NodeError> {
    items
        .iter()
        .map(|item| {
            if let Some(required) = only {
                if type_of(item)? != required {
                    return bad(format!("every child must be a `{required}` node"));
                }
            }
            from_json_value(item)
        })
        .collect()
}

fn from_json_value(json: &Json) -> Result<Node, NodeError> {
    let name = type_of(json)?;
    let kind = match name {
        "text" => {
            let f = Fields::of(json, "text", &["type", "text"])?;
            NodeKind::Text(f.string("text")?)
        }
        "table" => {
            let f = Fields::of(
                json,
                "table",
                &["type", "title", "caption", "columns", "rows"],
            )?;
            let mut columns = Vec::new();
            for column in f.array("columns")? {
                let c = Fields::of(column, "column", &["header", "justify"])?;
                columns.push((c.string("header")?, justify_of(&c.string("justify")?)?));
            }
            let mut rows = Vec::new();
            for row in f.array("rows")? {
                let Json::Arr(cells) = row else {
                    return bad("a row must be an array");
                };
                rows.push(
                    cells
                        .iter()
                        .map(|cell| match cell {
                            Json::Str(s) => Ok(s.clone()),
                            _ => bad("a cell must be a string"),
                        })
                        .collect::<Result<Vec<_>, _>>()?,
                );
            }
            NodeKind::Table {
                title: f.optional("title")?,
                caption: f.optional("caption")?,
                columns,
                rows,
            }
        }
        "panel" => {
            let f = Fields::of(json, "panel", &["type", "title", "subtitle", "body"])?;
            NodeKind::Panel {
                title: f.optional("title")?,
                subtitle: f.optional("subtitle")?,
                body: Box::new(from_json_value(f.get("body"))?),
            }
        }
        "tree" => {
            let f = Fields::of(json, "tree", &["type", "label", "children"])?;
            NodeKind::Tree {
                label: f.string("label")?,
                children: node_list(f.array("children")?, Some("tree"))?,
            }
        }
        "progress" => {
            let f = Fields::of(json, "progress", &["type", "tasks"])?;
            let mut tasks = Vec::new();
            for task in f.array("tasks")? {
                let t = Fields::of(
                    task,
                    "task",
                    &["description", "completed", "total", "finished", "visible"],
                )?;
                tasks.push(TaskState {
                    description: t.string("description")?,
                    completed: t.int("completed")?,
                    total: t.int("total")?,
                    finished: t.boolean("finished")?,
                    visible: t.boolean("visible")?,
                });
            }
            NodeKind::Progress(tasks)
        }
        "error" => {
            let f = Fields::of(json, "error", &["type", "message", "causes", "hint"])?;
            NodeKind::Error {
                message: f.string("message")?,
                causes: f.strings("causes")?,
                hint: f.optional("hint")?,
            }
        }
        "columns" => {
            let f = Fields::of(json, "columns", &["type", "title", "items"])?;
            NodeKind::Columns {
                title: f.optional("title")?,
                items: node_list(f.array("items")?, None)?,
            }
        }
        "layout" => {
            let f = Fields::of(
                json,
                "layout",
                &[
                    "type",
                    "name",
                    "ratio",
                    "size",
                    "visible",
                    "direction",
                    "content",
                    "children",
                ],
            )?;
            let content = match f.get("content") {
                Json::Null => None,
                other => Some(Box::new(from_json_value(other)?)),
            };
            NodeKind::Layout {
                name: f.optional("name")?,
                ratio: f.int("ratio")?,
                size: f.optional_int("size")?,
                visible: f.boolean("visible")?,
                direction: direction_of(&f.string("direction")?)?,
                content,
                children: node_list(f.array("children")?, Some("layout"))?,
            }
        }
        "group" => {
            let f = Fields::of(json, "group", &["type", "items"])?;
            NodeKind::Group(node_list(f.array("items")?, None)?)
        }
        "padding" => {
            let f = Fields::of(
                json,
                "padding",
                &["type", "top", "right", "bottom", "left", "content"],
            )?;
            NodeKind::Padding {
                top: f.int("top")?,
                right: f.int("right")?,
                bottom: f.int("bottom")?,
                left: f.int("left")?,
                content: Box::new(from_json_value(f.get("content"))?),
            }
        }
        other => return bad(format!("unknown node type `{other}`")),
    };
    Ok(Node(kind))
}

// ---------------------------------------------------------------------------
// The public surface
// ---------------------------------------------------------------------------

impl Node {
    /// The document `hud/1` for this node: JSON in the canonical form (two space indent, keys in
    /// the order of the schema, a final newline), the same bytes on every run and every console.
    ///
    /// ```
    /// use hud::Node;
    ///
    /// assert_eq!(
    ///     Node::text("ok").to_json(),
    ///     "{\n  \"schema\": \"hud/1\",\n  \"content\": {\n    \"type\": \"text\",\n    \"text\": \"ok\"\n  }\n}\n"
    /// );
    /// ```
    pub fn to_json(&self) -> String {
        let document = object(vec![
            ("schema", string("hud/1")),
            ("content", to_json_value(self)),
        ]);
        let mut out = String::new();
        write(&mut out, &document, 0);
        out.push('\n');
        out
    }

    /// Reads a document `hud/1`, such as [`Node::to_json`] writes. Any JSON whitespace is
    /// accepted; the keys of every node must be exactly the ones of its type.
    ///
    /// ```
    /// use hud::Node;
    ///
    /// assert!(Node::from_json("{\"schema\": \"hud/2\", \"content\": null}").is_err());
    /// ```
    pub fn from_json(text: &str) -> Result<Node, NodeError> {
        let json = parse(text)?;
        let f = Fields::of(&json, "document", &["schema", "content"])?;
        if f.string("schema")? != "hud/1" {
            return bad("the schema must be `hud/1`");
        }
        from_json_value(f.get("content"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table() -> Node {
        Node(NodeKind::Table {
            title: Some("T".into()),
            caption: None,
            columns: vec![
                ("Name".into(), Justify::Left),
                ("Count".into(), Justify::Right),
            ],
            rows: vec![vec!["api".into(), "12".into()], vec![]],
        })
    }

    #[test]
    fn a_node_survives_a_round_trip_byte_for_byte() {
        let node = Node::group([
            table(),
            Node::text("tab\t \"quote\" \\ \u{1} é 日本 🚀"),
            Node(NodeKind::Layout {
                name: Some("root".into()),
                ratio: 1,
                size: None,
                visible: true,
                direction: Direction::Row,
                content: None,
                children: vec![Node(NodeKind::Layout {
                    name: None,
                    ratio: 2,
                    size: Some(5),
                    visible: false,
                    direction: Direction::None,
                    content: Some(Box::new(Node::text("x"))),
                    children: vec![],
                })],
            }),
        ]);
        let json = node.to_json();
        let back = Node::from_json(&json).unwrap();
        assert_eq!(back, node);
        assert_eq!(back.to_json(), json);
    }

    #[test]
    fn the_canonical_form_matches_python_dumps_with_indent_two() {
        let json = Node::text("a\u{1f}b").to_json();
        assert_eq!(
            json,
            "{\n  \"schema\": \"hud/1\",\n  \"content\": {\n    \"type\": \"text\",\n    \"text\": \"a\\u001fb\"\n  }\n}\n"
        );
        let empty = Node::group([]).to_json();
        assert!(empty.contains("\"items\": []"));
    }

    #[test]
    fn unicode_escapes_and_surrogate_pairs_are_read() {
        let json = "{\"schema\":\"hud/1\",\"content\":{\"type\":\"text\",\"text\":\"\\ud83d\\ude80 \\u00e9\"}}";
        assert_eq!(Node::from_json(json).unwrap(), Node::text("🚀 é"));
    }

    #[test]
    fn documents_that_are_not_hud_1_are_refused() {
        for bad in [
            "",
            "{",
            "[]",
            "{\"schema\":\"hud/1\"}",
            "{\"schema\":\"hud/2\",\"content\":{\"type\":\"text\",\"text\":\"x\"}}",
            "{\"schema\":\"hud/1\",\"content\":{\"type\":\"text\",\"text\":\"x\",\"extra\":1}}",
            "{\"schema\":\"hud/1\",\"content\":{\"type\":\"text\",\"text\":1.5}}",
            "{\"schema\":\"hud/1\",\"content\":{\"type\":\"text\",\"text\":-1}}",
            "{\"schema\":\"hud/1\",\"content\":{\"type\":\"nope\"}}",
            "{\"schema\":\"hud/1\",\"content\":{\"type\":\"text\",\"text\":\"\\ud800\"}}",
            "{\"schema\":\"hud/1\",\"content\":{\"type\":\"text\",\"text\":\"x\"}} x",
            "{\"schema\":\"hud/1\",\"schema\":\"hud/1\",\"content\":null}",
        ] {
            assert!(Node::from_json(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn nesting_beyond_the_limit_is_an_error_not_a_stack_overflow() {
        let deep = "[".repeat(10_000);
        assert!(Node::from_json(&deep).is_err());
    }

    #[test]
    fn tree_children_must_be_trees() {
        let json = "{\"schema\":\"hud/1\",\"content\":{\"type\":\"tree\",\"label\":\"a\",\"children\":[{\"type\":\"text\",\"text\":\"b\"}]}}";
        assert!(Node::from_json(json).is_err());
    }
}
