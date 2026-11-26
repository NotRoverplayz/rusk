
use super::parser::{Element, Node};
use super::web_scripting::WebRuskCompiler;
use crate::lexer::Lexer as RuskLexer;
use crate::parser::Parser as RuskParser;

pub struct WebRenderer;

impl WebRenderer {
    #[allow(dead_code)]
    pub fn render(nodes: &[Node]) -> String {
        Self::render_with_rusk(nodes, None)
    }

    pub fn render_with_rusk(nodes: &[Node], rusk_source: Option<&str>) -> String {
        let mut output = String::new();

        let title = Self::find_title(nodes)
            .unwrap_or_else(|| "RUSK Website".to_string());

        output.push_str("<!DOCTYPE html>\n");
        output.push_str("<html lang=\"en\">\n<head>\n");
        output.push_str("<meta charset=\"UTF-8\">\n");
        output.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");

        output.push_str("<title>");
        output.push_str(&Self::escape_html(&title));
        output.push_str("</title>\n");

        output.push_str("<style>\n");
        output.push_str("* { box-sizing: border-box; }\n");
        output.push_str("body { margin: 0; font-family: Arial, sans-serif; }\n");
        output.push_str("button { cursor: pointer; border: none; padding: 12px 24px; border-radius: 8px; }\n");
        output.push_str(".rusk-row { display: flex; flex-direction: row; }\n");
        output.push_str(".rusk-column { display: flex; flex-direction: column; }\n");
        output.push_str(".rusk-center { display: flex; justify-content: center; align-items: center; }\n");
        output.push_str(".rusk-container { width: 100%; max-width: 1200px; margin-left: auto; margin-right: auto; }\n");
        output.push_str(".rusk-grid { display: grid; }\n");
        output.push_str(".rusk-spacer { flex: 1; }\n");
        output.push_str(".rusk-text { margin: 0; }\n");
        output.push_str("@keyframes rusk-fade-in { from { opacity: 0; } to { opacity: 1; } }\n");
        output.push_str("@keyframes rusk-fade-out { from { opacity: 1; } to { opacity: 0; } }\n");
        output.push_str("@keyframes rusk-slide { from { transform: translateY(24px); opacity: 0; } to { transform: translateY(0); opacity: 1; } }\n");
        output.push_str("@keyframes rusk-scale { from { transform: scale(.85); } to { transform: scale(1); } }\n");
        output.push_str("@keyframes rusk-bounce { 0%,100% { transform: translateY(0); } 50% { transform: translateY(-12px); } }\n");
        output.push_str("@keyframes rusk-rotate { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }\n");
        output.push_str(Self::responsive_css(nodes).as_str());
        output.push_str("</style>\n");
        output.push_str(Self::runtime_script());
        if let Some(source) = rusk_source {
            match RuskLexer::new(source).tokenize()
                .and_then(|tokens| RuskParser::new(tokens).parse())
                .and_then(|program| WebRuskCompiler::compile(&program))
            {
                Ok(script) => {
                    output.push_str("\n<script>\n");
                    output.push_str(&script);
                    output.push_str("\n</script>");
                }
                Err(error) => {
                    let message = Self::escape_html(&error);
                    output.push_str("\n<script>\nconsole.error(");
                    output.push_str(&Self::escape_js_string(&message));
                    output.push_str(");\n</script>");
                }
            }
        }
        output.push_str("\n</head>\n<body>\n");

        for node in nodes {
            output.push_str(&Self::render_document_node(node));
        }

        output.push_str("\n</body>\n</html>");

        output
    }

    fn find_title(nodes: &[Node]) -> Option<String> {
        for node in nodes {
            if let Node::Element(element) = node {
                if element.name == "window" {
                    if let Some((_, value)) =
                        element.attributes.iter().find(|(name, _)| name == "title")
                    {
                        return Some(value.clone());
                    }
                }

                if let Some(title) = Self::find_title(&element.children) {
                    return Some(title);
                }
            }
        }

        None
    }

    fn render_node(node: &Node) -> String {
        match node {
            Node::Text(text) => Self::escape_html(text),
            Node::Element(element) => Self::render_element(element),
        }
    }

    fn render_element(element: &Element) -> String {
        if element.name == "window" {
            let mut output = String::new();

            for child in &element.children {
                output.push_str(&Self::render_node(child));
            }

            return output;
        }

        let html_tag = match element.name.as_str() {
            "container" | "row" | "column" | "center" | "grid" | "spacer" => "div",
            "text" | "paragraph" => "p",
            "heading" => "h2",
            "section" => "section",
            "header" => "header",
            "footer" => "footer",
            "nav" => "nav",
            "card" => "article",
            "modal" => "dialog",
            "table" => "table",
            "divider" => "hr",
            "progress" => "progress",
            "loading" => "div",
            "button" => "button",
            "image" => "img",
            "link" => "a",
            "input" => "input",
            "textarea" => "textarea",
            "checkbox" => "input",
            "select" => "select",
            "form" => "form",
            "list" => "ul",
            "icon" => "span",
            "video" => "video",
            other => other,
        };

        let mut output = String::new();
        let mut styles = Vec::new();
        let mut classes = Vec::new();

        // Default layout behavior
        match element.name.as_str() {
            "row" => classes.push("rusk-row"),
            "column" => classes.push("rusk-column"),
            "center" => classes.push("rusk-center"),
            "container" => classes.push("rusk-container"),
            "grid" => classes.push("rusk-grid"),
            "spacer" => classes.push("rusk-spacer"),
            _ => {}
        }

        for (name, value) in &element.attributes {
            match name.as_str() {
                "background" => {
                    if Self::valid_css_value(value) {
                        styles.push(format!("background-color: {}", value));
                    }
                }
                "color" => {
                    if html_tag == "button" {
                        styles.push(format!("background-color: {}", value));
                        styles.push("color: white".to_string());
                    } else {
                        styles.push(format!("color: {}", value));
                    }
                }
                "size" => {
                    let size = if value.chars().all(|c| c.is_ascii_digit()) {
                        format!("{}px", value)
                    } else {
                        value.clone()
                    };
                    styles.push(format!("font-size: {}", size));
                }
                "font" => styles.push(format!("font-family: {}", value)),
                "align" => {
                    let align = match value.as_str() {
                        "center" => "center",
                        "start" | "left" => "flex-start",
                        "end" | "right" => "flex-end",
                        "stretch" => "stretch",
                        other => other,
                    };

                    if matches!(
                        element.name.as_str(),
                        "row" | "column" | "center"
                    ) {
                        styles.push(format!("align-items: {}", align));
                    } else {
                        styles.push(format!("text-align: {}", value));
                    }
                }
                "justify" => {
                    let justify = match value.as_str() {
                        "start" | "left" => "flex-start",
                        "end" | "right" => "flex-end",
                        "center" => "center",
                        "between" => "space-between",
                        "around" => "space-around",
                        "evenly" => "space-evenly",
                        other => other,
                    };

                    styles.push(format!("justify-content: {}", justify));
                }
                "gap" => styles.push(format!("gap: {}", value)),
                "radius" => styles.push(format!("border-radius: {}", value)),
                "shadow" => {
                    let shadow = match value.as_str() {
                        "small" => "0 2px 8px rgba(0,0,0,.18)",
                        "medium" => "0 8px 24px rgba(0,0,0,.22)",
                        "large" => "0 16px 48px rgba(0,0,0,.28)",
                        custom => custom,
                    };
                    styles.push(format!("box-shadow: {}", shadow));
                }
                "opacity" => styles.push(format!("opacity: {}", value)),
                "position" => styles.push(format!("position: {}", value)),
                "overflow" => styles.push(format!("overflow: {}", value)),
                "visibility" => styles.push(format!("visibility: {}", value)),
                "transform" => styles.push(format!("transform: {}", value)),
                "transition" => styles.push(format!("transition: {}", value)),
                "duration" => {}
                "delay" => {}
                "loop" => {}
                "animation" => {
                    if let Some((name, default_duration)) = match value.as_str() {
                        "fade-in" => Some(("rusk-fade-in", "500ms")),
                        "fade-out" => Some(("rusk-fade-out", "500ms")),
                        "slide" => Some(("rusk-slide", "500ms")),
                        "scale" => Some(("rusk-scale", "500ms")),
                        "bounce" => Some(("rusk-bounce", "700ms")),
                        "rotate" => Some(("rusk-rotate", "1000ms")),
                        _ => None,
                    } {
                        let duration = element.attribute("duration").unwrap_or(default_duration);
                        let delay = element.attribute("delay").unwrap_or("0ms");
                        let iteration = element.attribute("loop").map_or("1", |v| {
                            if v == "true" { "infinite" } else { v }
                        });
                        styles.push(format!(
                            "animation: {} {} ease both {} {}",
                            name, duration, delay, iteration
                        ));
                    }
                }
                "mobile-columns" | "tablet-columns" => {}
                "wrap" if value == "true" => {
                    styles.push("flex-wrap: wrap".to_string());
                }
                "wrap" if value == "false" => {
                    styles.push("flex-wrap: nowrap".to_string());
                }
                "columns" => {
                    let columns = if value.chars().all(|c| c.is_ascii_digit()) {
                        format!("repeat({}, minmax(0, 1fr))", value)
                    } else {
                        value.clone()
                    };

                    styles.push(format!("grid-template-columns: {}", columns));
                }
                "padding" | "margin" | "width" | "height"
                | "border" | "border-radius" | "display"
                | "max-width" | "min-width" | "max-height"
                | "min-height" => {
                    styles.push(format!("{}: {}", name, value));
                }
                _ => {}
            }
        }

        output.push('<');
        output.push_str(html_tag);

        if element.name == "checkbox" {
            output.push_str(" type=\"checkbox\"");
        }

        if let Some(bind) = element.attribute("bind") {
            if Self::valid_identifier(bind) {
                output.push_str(" data-rusk-bind=\"");
                output.push_str(bind);
                output.push('"');
            }
        }

        for (name, value) in &element.attributes {
            if matches!(
                name.as_str(),
                "on-click" | "on-hover" | "on-focus" | "on-change"
                    | "on-submit" | "on-input" | "on-load"
                    | "on-scroll" | "on-keydown" | "on-keyup"
            ) {
                if Self::valid_identifier(value) {
                    output.push_str(" data-rusk-");
                    output.push_str(name);
                    output.push_str("=\"");
                    output.push_str(&Self::escape_html(value));
                    output.push('"');
                }
            }
        }

        if element.name == "grid" {
            if let Some(value) = element.attribute("mobile-columns") {
                if value.parse::<u16>().is_ok() {
                    output.push_str(" data-mobile-columns=\"");
                    output.push_str(value);
                    output.push('"');
                }
            }
            if let Some(value) = element.attribute("tablet-columns") {
                if value.parse::<u16>().is_ok() {
                    output.push_str(" data-tablet-columns=\"");
                    output.push_str(value);
                    output.push('"');
                }
            }
        }

        for (name, value) in &element.attributes {
            if matches!(
                name.as_str(),
                "background" | "color" | "size" | "font"
                    | "align" | "justify" | "gap" | "wrap"
                    | "columns" | "padding" | "margin" | "width"
                    | "height" | "border" | "border-radius"
                    | "display" | "max-width" | "min-width"
                    | "max-height" | "min-height" | "title"
                    | "radius" | "shadow" | "opacity" | "position"
                    | "overflow" | "visibility" | "transform"
                    | "transition" | "animation" | "duration"
                    | "delay" | "loop" | "mobile-columns"
                        | "tablet-columns" | "bind" | "script"
                    | "on-click" | "on-hover" | "on-focus"
                    | "on-change" | "on-submit" | "on-input"
                    | "on-load" | "on-scroll" | "on-keydown"
                    | "on-keyup"
            ) {
                continue;
            }

            if !name.chars().all(|c| {
                c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == ':'
            }) {
                continue;
            }

            if name == "class" {
                classes.push(value.as_str());
                continue;
            }

            output.push(' ');
            output.push_str(name);
            output.push_str("=\"");
            output.push_str(&Self::escape_html(value));
            output.push('"');
        }

        if !classes.is_empty() {
            output.push_str(" class=\"");
            output.push_str(&Self::escape_html(
                &classes.join(" ")
            ));
            output.push('"');
        }

        if !styles.is_empty() {
            output.push_str(" style=\"");
            let safe_styles: Vec<String> = styles
                .into_iter()
                .filter(|style| style.split_once(':').is_some_and(|(_, value)| Self::valid_css_value(value)))
                .collect();
            output.push_str(&Self::escape_html(&safe_styles.join("; ")));
            output.push('"');
        }

        if matches!(html_tag, "img" | "input" | "hr") {
            output.push_str(" />");
            return output;
        }

        output.push('>');

        for child in &element.children {
            output.push_str(&Self::render_node(child));
        }

        output.push_str("</");
        output.push_str(html_tag);
        output.push('>');

        output
    }

    fn render_document_node(node: &Node) -> String {
        match node {
            Node::Element(element) if element.name == "window" => {
                let mut output = String::new();
                for child in &element.children {
                    match child {
                        Node::Element(child_element) if child_element.name == "body" => {
                            output.push_str(&Self::render_body(child_element));
                        }
                        Node::Element(child_element) if child_element.name == "head" => {}
                        _ => output.push_str(&Self::render_node(child)),
                    }
                }
                output
            }
            _ => Self::render_node(node),
        }
    }

    fn render_body(body: &Element) -> String {
        let mut output = String::new();
        output.push_str("<main");
        let mut style = Vec::new();
        for (name, value) in &body.attributes {
            match name.as_str() {
                "background" => style.push(format!("background-color: {}", value)),
                "color" => style.push(format!("color: {}", value)),
                "padding" | "margin" | "width" | "height" | "min-height" | "max-height" => {
                    style.push(format!("{}: {}", name, value));
                }
                _ => {}
            }
        }
        if !style.is_empty() {
            output.push_str(" style=\"");
            output.push_str(&Self::escape_html(&style.join("; ")));
            output.push('"');
        }
        output.push('>');
        for child in &body.children {
            output.push_str(&Self::render_node(child));
        }
        output.push_str("</main>");
        output
    }

    fn responsive_css(nodes: &[Node]) -> String {
        let mut mobile = std::collections::BTreeSet::new();
        let mut tablet = std::collections::BTreeSet::new();
        Self::collect_responsive(nodes, &mut mobile, &mut tablet);
        let mut css = String::new();
        for columns in mobile {
            css.push_str(&format!(
                "@media (max-width: 767px) {{ .rusk-grid[data-mobile-columns=\"{}\"] {{ grid-template-columns: repeat({}, minmax(0, 1fr)); }} }}\n",
                columns, columns
            ));
        }
        for columns in tablet {
            css.push_str(&format!(
                "@media (min-width: 768px) and (max-width: 1024px) {{ .rusk-grid[data-tablet-columns=\"{}\"] {{ grid-template-columns: repeat({}, minmax(0, 1fr)); }} }}\n",
                columns, columns
            ));
        }
        css
    }

    fn collect_responsive(
        nodes: &[Node],
        mobile: &mut std::collections::BTreeSet<String>,
        tablet: &mut std::collections::BTreeSet<String>,
    ) {
        for node in nodes {
            if let Node::Element(element) = node {
                if element.name == "grid" {
                    if let Some(value) = element.attribute("mobile-columns") {
                        if value.parse::<u16>().is_ok() {
                            mobile.insert(value.to_string());
                        }
                    }
                    if let Some(value) = element.attribute("tablet-columns") {
                        if value.parse::<u16>().is_ok() {
                            tablet.insert(value.to_string());
                        }
                    }
                }
                Self::collect_responsive(&element.children, mobile, tablet);
            }
        }
    }

    fn runtime_script() -> &'static str {
        r#"<script>
(() => {
  const validHandler = /^[A-Za-z_][A-Za-z0-9_]*$/;
  const handlers = ["on-click","on-hover","on-focus","on-change","on-submit","on-input","on-load","on-scroll","on-keydown","on-keyup"];
  window.RuskWeb = window.RuskWeb || {
    state: {},
    setState(name, value) {
      if (!/^[A-Za-z_][A-Za-z0-9_]*$/.test(name)) return;
      this.state[name] = value;
      document.querySelectorAll("*").forEach(el => {
        if (el.getAttribute("data-rusk-bind") !== name) return;
        if (el instanceof HTMLInputElement) {
          if (el.type === "checkbox") el.checked = Boolean(value);
          else el.value = String(value);
        } else if (el instanceof HTMLTextAreaElement) {
          el.value = String(value);
        } else if (el instanceof HTMLSelectElement) {
          el.value = String(value);
        } else {
          el.textContent = String(value);
        }
      });
    },
    call(name, payload) {
      if (!validHandler.test(name)) return;
      if (window.RuskApp && typeof window.RuskApp[name] === "function") {
        try { window.RuskApp[name](payload || {}); }
        catch (error) { console.error(`RUSK runtime error in '${name}':`, error instanceof Error ? error.message : error); }
      } else {
        console.warn(`RUSK web handler not registered: ${name}`);
      }
    }
  };
  const eventMap = { "on-click":"click","on-hover":"mouseover","on-focus":"focus","on-change":"change","on-submit":"submit","on-input":"input","on-load":"load","on-scroll":"scroll","on-keydown":"keydown","on-keyup":"keyup" };
  const registerEvents = () => {
  document.querySelectorAll("*").forEach(el => {
    handlers.forEach(name => {
      const handler = el.dataset["rusk" + name.split("-").map(part => part[0].toUpperCase() + part.slice(1)).join("")];
      if (!handler || !validHandler.test(handler)) return;
      el.addEventListener(eventMap[name], event => {
        if (name === "on-submit") event.preventDefault();
        const payload = { event, element: el };
        if (el instanceof HTMLInputElement) payload.value = el.type === "checkbox" ? el.checked : el.value;
        else if (el instanceof HTMLTextAreaElement || el instanceof HTMLSelectElement) payload.value = el.value;
        RuskWeb.call(handler, payload);
      });
    });
  });
  document.querySelectorAll("[data-rusk-bind]").forEach(el => {
    const name = el.getAttribute("data-rusk-bind");
    if (Object.prototype.hasOwnProperty.call(RuskWeb.state, name)) {
      RuskWeb.setState(name, RuskWeb.state[name]);
    }
  });
  };
  if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", registerEvents, { once: true });
  else registerEvents();
})();
</script>"#
    }

    fn valid_identifier(value: &str) -> bool {
        let mut chars = value.chars();
        matches!(chars.next(), Some(c) if c == '_' || c.is_ascii_alphabetic())
            && chars.all(|c| c == '_' || c.is_ascii_alphanumeric())
    }

    fn valid_css_value(value: &str) -> bool {
        let lower = value.to_ascii_lowercase();
        !value.is_empty()
            && !value.contains(['<', '>', '{', '}', ';', '\0'])
            && !lower.contains("javascript:")
            && !lower.contains("expression(")
            && !lower.contains("@import")
    }

    fn escape_html(value: &str) -> String {
        value
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#39;")
    }

    fn escape_js_string(value: &str) -> String {
        let mut output = String::from("\"");
        for character in value.chars() {
            match character {
                '"' => output.push_str("\\\""),
                '\\' => output.push_str("\\\\"),
                '\n' => output.push_str("\\n"),
                '\r' => output.push_str("\\r"),
                c if c.is_control() => output.push_str(&format!("\\u{:04x}", c as u32)),
                c => output.push(c),
            }
        }
        output.push('"');
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_rsml_to_html() {
        let element = Element {
            name: "text".to_string(),
            attributes: vec![
                ("class".to_string(), "greeting".to_string()),
            ],
            children: vec![
                Node::Text("Hello from RUSK!".to_string()),
            ],
        };

        let html = WebRenderer::render(&[Node::Element(element)]);

        assert!(html.contains("<p"));
        assert!(html.contains("class=\"greeting\""));
        assert!(html.contains("Hello from RUSK!"));
    }

    #[test]
    fn renders_styling_attributes() {
        let element = Element {
            name: "text".to_string(),
            attributes: vec![
                ("color".to_string(), "purple".to_string()),
                ("size".to_string(), "32".to_string()),
            ],
            children: vec![
                Node::Text("Styled text".to_string()),
            ],
        };

        let html = WebRenderer::render(&[Node::Element(element)]);

        assert!(html.contains("color: purple"));
        assert!(html.contains("font-size: 32px"));
    }

    #[test]
    fn renders_layout_elements() {
        let element = Element {
            name: "row".to_string(),
            attributes: vec![
                ("gap".to_string(), "20px".to_string()),
                ("justify".to_string(), "center".to_string()),
            ],
            children: vec![],
        };

        let html = WebRenderer::render(&[Node::Element(element)]);

        assert!(html.contains("rusk-row"));
        assert!(html.contains("gap: 20px"));
        assert!(html.contains("justify-content: center"));
    }

    #[test]
    fn renders_grid_columns() {
        let element = Element {
            name: "grid".to_string(),
            attributes: vec![
                ("columns".to_string(), "3".to_string()),
            ],
            children: vec![],
        };

        let html = WebRenderer::render(&[Node::Element(element)]);

        assert!(html.contains("rusk-grid"));
        assert!(html.contains("repeat(3, minmax(0, 1fr))"));
    }

    #[test]
    fn reads_window_title() {
        let element = Element {
            name: "window".to_string(),
            attributes: vec![
                ("title".to_string(), "My RUSK Website".to_string()),
            ],
            children: vec![],
        };

        let html = WebRenderer::render(&[Node::Element(element)]);

        assert!(html.contains("<title>My RUSK Website</title>"));
    }

    #[test]
    fn renders_safe_event_and_bind_bridge() {
        let element = Element {
            name: "button".to_string(),
            attributes: vec![
                ("on-click".to_string(), "increment".to_string()),
                ("bind".to_string(), "count".to_string()),
            ],
            children: vec![Node::Text("Increment".to_string())],
        };

        let html = WebRenderer::render(&[Node::Element(element)]);

        assert!(html.contains("data-rusk-on-click=\"increment\""));
        assert!(html.contains("data-rusk-bind=\"count\""));
        assert!(html.contains("window.RuskWeb"));
    }

    #[test]
    fn renders_responsive_grid_breakpoints() {
        let element = Element {
            name: "grid".to_string(),
            attributes: vec![
                ("columns".to_string(), "3".to_string()),
                ("tablet-columns".to_string(), "2".to_string()),
                ("mobile-columns".to_string(), "1".to_string()),
            ],
            children: vec![],
        };

        let html = WebRenderer::render(&[Node::Element(element)]);

        assert!(html.contains("data-mobile-columns=\"1\""));
        assert!(html.contains("data-tablet-columns=\"2\""));
        assert!(html.contains("@media (max-width: 767px)"));
        assert!(html.contains("@media (min-width: 768px)"));
    }

    #[test]
    fn embeds_native_rusk_web_script() {
        let source = r#"let counter be 0
function increment() {
    set counter to counter + 1
}"#;
        let html = WebRenderer::render_with_rusk(&[], Some(source));

        assert!(html.contains("ruskBind(\"counter\", 0"));
        assert!(html.contains("ruskFunctions[\"increment\"]"));
        assert!(html.contains("window.RuskApp"));
        assert!(html.contains("DOMContentLoaded"));
    }
}
