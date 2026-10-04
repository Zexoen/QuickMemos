use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

fn esc_text(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn esc_attr(s: &str) -> String {
    esc_text(s).replace('"', "&quot;").replace('\'', "&#39;")
}

#[derive(Clone, Copy)]
enum ItemKind {
    Ordered(u64),
    Bullet,
}

/// Render markdown as Pango markup suitable for a `gtk::Label`.
pub fn render_markdown(source: &str) -> String {
    let mut out = String::new();
    let mut items: Vec<ItemKind> = Vec::new();
    let mut quote_depth = 0usize;
    let mut item_bullet_start = None;

    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_TABLES);

    let parser = Parser::new_ext(source, options);
    for event in parser {
        match event {
            Event::Start(Tag::Paragraph) => {
                if quote_depth > 0 {
                    out.push_str("<span foreground=\"#8b949e\">▍ </span>");
                }
            }
            Event::End(TagEnd::Paragraph) => out.push_str("\n\n"),

            Event::Start(Tag::Heading { level, .. }) => {
                let size = match level {
                    pulldown_cmark::HeadingLevel::H1 => "175%",
                    pulldown_cmark::HeadingLevel::H2 => "150%",
                    pulldown_cmark::HeadingLevel::H3 => "135%",
                    pulldown_cmark::HeadingLevel::H4 => "120%",
                    pulldown_cmark::HeadingLevel::H5 => "110%",
                    pulldown_cmark::HeadingLevel::H6 => "105%",
                };
                out.push_str(&format!(
                    "<span font_weight=\"bold\" font_size=\"{size}\">"
                ));
            }
            Event::End(TagEnd::Heading(_)) => out.push_str("</span>\n\n"),

            Event::Start(Tag::BlockQuote(_)) => quote_depth += 1,
            Event::End(TagEnd::BlockQuote(_)) => {
                quote_depth = quote_depth.saturating_sub(1);
                out.push_str("\n\n");
            }

            Event::Start(Tag::CodeBlock(_)) => {
                out.push_str("<span font_family=\"monospace\" background=\"#eceef1\">");
            }
            Event::End(TagEnd::CodeBlock) => out.push_str("</span>\n\n"),

            Event::Start(Tag::List(start)) => {
                let kind = match start {
                    Some(n) => ItemKind::Ordered(n),
                    None => ItemKind::Bullet,
                };
                items.push(kind);
                let _ = start;
            }
            Event::End(TagEnd::List(tight)) => {
                items.pop();
                out.push_str(if tight { "\n" } else { "\n\n" });
            }

            Event::Start(Tag::Item) => {
                item_bullet_start = Some(out.len());
                match items.last() {
                    Some(ItemKind::Ordered(n)) => out.push_str(&format!("{n}. ")),
                    _ => out.push_str("•  "),
                }
            }
            Event::End(TagEnd::Item) => {
                item_bullet_start = None;
                out.push('\n');
                if let Some(ItemKind::Ordered(n)) = items.last_mut() {
                    *n += 1;
                }
            }

            Event::TaskListMarker(checked) => {
                if let Some(start) = item_bullet_start.take() {
                    out.truncate(start);
                }
                out.push_str(if checked { "<span foreground=\"#34a853\">☑</span> " } else { "☐ " });
            }

            Event::Start(Tag::Emphasis) => out.push_str("<i>"),
            Event::End(TagEnd::Emphasis) => out.push_str("</i>"),
            Event::Start(Tag::Strong) => out.push_str("<b>"),
            Event::End(TagEnd::Strong) => out.push_str("</b>"),
            Event::Start(Tag::Strikethrough) => out.push_str("<s>"),
            Event::End(TagEnd::Strikethrough) => out.push_str("</s>"),

            Event::Start(Tag::Link { dest_url, title, .. }) => {
                let mut a = format!("<a href=\"{}\"", esc_attr(&dest_url));
                if !title.is_empty() {
                    a.push_str(&format!(" title=\"{}\"", esc_attr(&title)));
                }
                a.push('>');
                out.push_str(&a);
            }
            Event::End(TagEnd::Link) => out.push_str("</a>"),

            Event::Start(Tag::Image { .. }) => out.push_str("<span foreground=\"#8b949e\">[img: "),
            Event::End(TagEnd::Image) => out.push_str("]</span>"),

            Event::Code(code) => {
                out.push_str(&format!(
                    "<span font_family=\"monospace\" background=\"#eceef1\">{}</span>",
                    esc_text(&code)
                ));
            }
            Event::Text(text) => out.push_str(&esc_text(&text)),
            Event::SoftBreak => out.push(' '),
            Event::HardBreak => out.push('\n'),
            Event::Rule => out.push_str("<span foreground=\"#c9ccd1\">————————</span>\n\n"),

            Event::Start(Tag::Table(_)) => out.push_str("<span font_family=\"monospace\">"),
            Event::End(TagEnd::Table) => out.push_str("</span>\n\n"),
            Event::Start(Tag::TableRow | Tag::TableHead) => out.push('\n'),
            Event::End(TagEnd::TableRow | TagEnd::TableHead) => out.push('\n'),
            Event::Start(Tag::TableCell) => out.push_str("| "),
            Event::End(TagEnd::TableCell) => out.push(' '),

            Event::Html(_) | Event::InlineHtml(_) => {}
            Event::FootnoteReference(_) => {}
            Event::InlineMath(m) | Event::DisplayMath(m) => {
                out.push_str(&format!(
                    "<span font_family=\"monospace\" foreground=\"#7b61ff\">{}</span>",
                    esc_text(&m)
                ));
            }

            Event::Start(_) | Event::End(_) => {}
        }
    }

    out.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check_well_formed(markup: &str) {
        let mut stack: Vec<&str> = Vec::new();
        let mut i = 0;
        while i < markup.len() {
            if markup.as_bytes()[i] == b'<' {
                let end = markup[i..]
                    .find('>')
                    .expect("unterminated tag in markup");
                let raw = &markup[i..=i + end];
                let inner = &markup[i + 1..i + end];
                if let Some(name) = inner.split_whitespace().next() {
                    let name = name.trim_end_matches('/');
                    if let Some(closed) = name.strip_prefix('/') {
                        assert_eq!(
                            stack.pop(),
                            Some(closed),
                            "mismatched close `{}` in `{}`",
                            raw,
                            markup
                        );
                    } else if !inner.ends_with('/') {
                        stack.push(name);
                    }
                }
                i += end + 1;
            } else {
                i += 1;
            }
        }
        assert!(stack.is_empty(), "unclosed tags {:?} in `{}`", stack, markup);
    }

    #[test]
    fn renders_headings_and_escapes() {
        let markup = render_markdown("# Hello & World");
        check_well_formed(&markup);
        assert!(markup.contains("font_weight=\"bold\""));
        assert!(markup.contains("Hello &amp; World"));
    }

    #[test]
    fn renders_inline_code_strong_emphasis_and_links() {
        let markup =
            render_markdown("Run `cargo build` for **real**, *fast*: [docs](https://example.com)");
        check_well_formed(&markup);
        assert!(markup.contains("font_family=\"monospace\""));
        assert!(markup.contains("<b>real</b>"));
        assert!(markup.contains("<i>fast</i>"));
        assert!(markup.contains("<a href=\"https://example.com\">docs</a>"));
    }

    #[test]
    fn renders_lists_and_task_markers() {
        let bullets = render_markdown("- one\n- two");
        check_well_formed(&bullets);
        assert_eq!(bullets.matches('•').count(), 2);

        let tasks = render_markdown("- [x] done\n- [ ] todo");
        check_well_formed(&tasks);
        assert!(tasks.contains("☑"));
        assert!(tasks.contains("☐"));
        assert!(!tasks.contains('•'));
    }

    #[test]
    fn escapes_link_href_attributes() {
        let markup = render_markdown("[x](https://example.com/?a=1&b=%22hi%22)");
        check_well_formed(&markup);
        assert!(markup.contains("href=\"https://example.com/?a=1&amp;b=%22hi%22\""));
    }

    #[test]
    fn empty_and_plain() {
        assert_eq!(render_markdown(""), "");
        assert_eq!(render_markdown("plain text"), "plain text");
    }
}