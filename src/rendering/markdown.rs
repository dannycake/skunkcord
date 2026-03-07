// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Discord-flavored markdown parser
//!
//! Parses Discord's variant of markdown into HTML for QML rendering.
//! Supports: bold, italic, underline, strikethrough, code, spoilers,
//! quotes, headings, mentions, custom emoji, timestamps, and links.
//!
//! # Example
//!
//! ```
//! use discord_qt::rendering::markdown::parse_markdown;
//!
//! let html = parse_markdown("**bold** and *italic*");
//! assert!(html.contains("<strong>bold</strong>"));
//! assert!(html.contains("<em>italic</em>"));
//! ```

use regex::Regex;

/// Parse Discord markdown to HTML for display
pub fn parse_markdown(input: &str) -> String {
    let mut result = input.to_string();

    // Order matters — process from most specific to least specific

    // Code blocks (``` ```) — must be first to avoid processing markdown inside code
    result = parse_code_blocks(&result);

    // Inline code (` `)
    result = parse_inline_code(&result);

    // Block quotes (> and >>>)
    result = parse_block_quotes(&result);

    // Headings (# ## ###)
    result = parse_headings(&result);

    // Spoilers (||text||)
    result = parse_spoilers(&result);

    // Bold italic (***text***)
    result = parse_bold_italic(&result);

    // Bold (**text**)
    result = parse_bold(&result);

    // Italic (*text* or _text_)
    result = parse_italic(&result);

    // Underline (__text__)
    result = parse_underline(&result);

    // Strikethrough (~~text~~)
    result = parse_strikethrough(&result);

    // Discord mentions (<@userid>, <#channelid>, <@&roleid>)
    result = parse_mentions(&result);

    // Custom emoji (<:name:id> and <a:name:id>)
    result = parse_custom_emoji(&result);

    // Discord timestamps (<t:1234567890:R>)
    result = parse_timestamps(&result);

    // Masked links ([text](url))
    result = parse_masked_links(&result);

    // Auto-link bare URLs
    result = parse_bare_urls(&result);

    // Newlines to <br>
    result = result.replace('\n', "<br>");

    result
}

fn parse_code_blocks(input: &str) -> String {
    let re = Regex::new(r"```(?:(\w+)\n)?([\s\S]*?)```").unwrap();
    re.replace_all(input, |caps: &regex::Captures| {
        let _lang = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let code = &caps[2];
        format!(
            "<pre style=\"background:#2b2d31;padding:8px;border-radius:4px;overflow-x:auto\"><code>{}</code></pre>",
            escape_html(code.trim())
        )
    })
    .to_string()
}

fn parse_inline_code(input: &str) -> String {
    let re = Regex::new(r"`([^`]+)`").unwrap();
    re.replace_all(input, |caps: &regex::Captures| {
        format!(
            "<code style=\"background:#2b2d31;padding:2px 4px;border-radius:3px\">{}</code>",
            escape_html(&caps[1])
        )
    })
    .to_string()
}

fn parse_block_quotes(input: &str) -> String {
    // >>> multiline quotes
    let re = Regex::new(r"(?m)^>>> (.+)$").unwrap();
    let result = re
        .replace_all(input, |caps: &regex::Captures| {
            format!(
                "<blockquote style=\"border-left:4px solid #4e5058;padding-left:12px;margin:4px 0\">{}</blockquote>",
                &caps[1]
            )
        })
        .to_string();

    // > single line quotes
    let re = Regex::new(r"(?m)^> (.+)$").unwrap();
    re.replace_all(&result, |caps: &regex::Captures| {
        format!(
            "<blockquote style=\"border-left:4px solid #4e5058;padding-left:12px;margin:2px 0\">{}</blockquote>",
            &caps[1]
        )
    })
    .to_string()
}

fn parse_headings(input: &str) -> String {
    let re = Regex::new(r"(?m)^(#{1,3})\s+(.+)$").unwrap();
    re.replace_all(input, |caps: &regex::Captures| {
        let level = caps[1].len();
        let size = match level {
            1 => "24px",
            2 => "20px",
            3 => "16px",
            _ => "16px",
        };
        format!(
            "<div style=\"font-size:{};font-weight:700;margin:8px 0\">{}</div>",
            size, &caps[2]
        )
    })
    .to_string()
}

fn parse_spoilers(input: &str) -> String {
    let re = Regex::new(r"\|\|(.+?)\|\|").unwrap();
    re.replace_all(input, |caps: &regex::Captures| {
        format!(
            "<span class=\"spoiler\" style=\"background:#1e1f22;color:transparent;border-radius:3px;padding:0 2px;cursor:pointer\">{}</span>",
            &caps[1]
        )
    })
    .to_string()
}

fn parse_bold_italic(input: &str) -> String {
    let re = Regex::new(r"\*\*\*(.+?)\*\*\*").unwrap();
    re.replace_all(input, "<strong><em>$1</em></strong>")
        .to_string()
}

fn parse_bold(input: &str) -> String {
    let re = Regex::new(r"\*\*(.+?)\*\*").unwrap();
    re.replace_all(input, "<strong>$1</strong>").to_string()
}

fn parse_italic(input: &str) -> String {
    // Simple approach: match *text* where text doesn't contain *
    // This runs after bold (** and ***) has already been processed
    let re = Regex::new(r"\*([^*]+?)\*").unwrap();
    re.replace_all(input, "<em>$1</em>").to_string()
}

fn parse_underline(input: &str) -> String {
    let re = Regex::new(r"__(.+?)__").unwrap();
    re.replace_all(input, "<u>$1</u>").to_string()
}

fn parse_strikethrough(input: &str) -> String {
    let re = Regex::new(r"~~(.+?)~~").unwrap();
    re.replace_all(input, "<s>$1</s>").to_string()
}

fn parse_mentions(input: &str) -> String {
    // User mention: <@123456> or <@!123456>
    let re = Regex::new(r"<@!?(\d+)>").unwrap();
    let result = re
        .replace_all(input, |caps: &regex::Captures| {
            format!(
                "<span class=\"mention\" style=\"background:#5865f233;color:#c9cdfb;padding:0 2px;border-radius:3px\">@user:{}</span>",
                &caps[1]
            )
        })
        .to_string();

    // Channel mention: <#123456>
    let re = Regex::new(r"<#(\d+)>").unwrap();
    let result = re
        .replace_all(&result, |caps: &regex::Captures| {
            format!(
                "<span class=\"mention\" style=\"background:#5865f233;color:#c9cdfb;padding:0 2px;border-radius:3px\">#channel:{}</span>",
                &caps[1]
            )
        })
        .to_string();

    // Role mention: <@&123456>
    let re = Regex::new(r"<@&(\d+)>").unwrap();
    re.replace_all(&result, |caps: &regex::Captures| {
        format!(
            "<span class=\"mention\" style=\"background:#5865f233;color:#c9cdfb;padding:0 2px;border-radius:3px\">@role:{}</span>",
            &caps[1]
        )
    })
    .to_string()
}

fn parse_custom_emoji(input: &str) -> String {
    // Animated: <a:name:id>
    let re = Regex::new(r"<a:(\w+):(\d+)>").unwrap();
    let result = re
        .replace_all(input, |caps: &regex::Captures| {
            format!(
                "<img src=\"https://cdn.discordapp.com/emojis/{}.gif\" alt=\":{}:\" class=\"emoji\" style=\"width:22px;height:22px;vertical-align:middle\">",
                &caps[2], &caps[1]
            )
        })
        .to_string();

    // Static: <:name:id>
    let re = Regex::new(r"<:(\w+):(\d+)>").unwrap();
    re.replace_all(&result, |caps: &regex::Captures| {
        format!(
            "<img src=\"https://cdn.discordapp.com/emojis/{}.png\" alt=\":{}:\" class=\"emoji\" style=\"width:22px;height:22px;vertical-align:middle\">",
            &caps[2], &caps[1]
        )
    })
    .to_string()
}

fn parse_timestamps(input: &str) -> String {
    // Discord timestamp: <t:1234567890:R> (various format styles)
    let re = Regex::new(r"<t:(\d+)(?::([tTdDfFR]))?>").unwrap();
    re.replace_all(input, |caps: &regex::Captures| {
        let timestamp = caps[1].parse::<i64>().unwrap_or(0);
        let style = caps.get(2).map(|m| m.as_str()).unwrap_or("f");
        format!(
            "<span class=\"timestamp\" data-timestamp=\"{}\" data-style=\"{}\" style=\"background:#3b3d44;padding:0 4px;border-radius:3px\">{}</span>",
            timestamp, style, format_timestamp(timestamp, style)
        )
    })
    .to_string()
}

fn format_timestamp(timestamp: i64, style: &str) -> String {
    use chrono::{TimeZone, Utc};
    let dt = Utc.timestamp_opt(timestamp, 0);
    match dt {
        chrono::LocalResult::Single(dt) => match style {
            "R" => format_relative_time(dt),
            "t" => dt.format("%H:%M").to_string(),
            "T" => dt.format("%H:%M:%S").to_string(),
            "d" => dt.format("%m/%d/%Y").to_string(),
            "D" => dt.format("%B %d, %Y").to_string(),
            "F" => dt.format("%A, %B %d, %Y %H:%M").to_string(),
            _ => dt.format("%B %d, %Y %H:%M").to_string(), // "f" default
        },
        _ => format!("<t:{}>", timestamp),
    }
}

fn format_relative_time(dt: chrono::DateTime<chrono::Utc>) -> String {
    let now = chrono::Utc::now();
    let diff = now.signed_duration_since(dt);
    let secs = diff.num_seconds().abs();

    let (value, unit) = if secs < 60 {
        (secs, "second")
    } else if secs < 3600 {
        (secs / 60, "minute")
    } else if secs < 86400 {
        (secs / 3600, "hour")
    } else if secs < 2592000 {
        (secs / 86400, "day")
    } else if secs < 31536000 {
        (secs / 2592000, "month")
    } else {
        (secs / 31536000, "year")
    };

    let plural = if value != 1 { "s" } else { "" };
    if diff.num_seconds() >= 0 {
        format!("{} {}{} ago", value, unit, plural)
    } else {
        format!("in {} {}{}", value, unit, plural)
    }
}

fn parse_masked_links(input: &str) -> String {
    // [display text](url)
    let re = Regex::new(r"\[([^\]]+)\]\((https?://[^\)]+)\)").unwrap();
    re.replace_all(input, |caps: &regex::Captures| {
        format!(
            "<a href=\"{}\" style=\"color:#00aff4\" title=\"{}\">{}</a>",
            &caps[2], &caps[2], &caps[1]
        )
    })
    .to_string()
}

fn parse_bare_urls(input: &str) -> String {
    // Skip if the URL is already inside an HTML tag (href="..." or src="...")
    // Simple approach: only link URLs that aren't preceded by = or "
    let mut result = String::new();
    let mut last_end = 0;
    let re = Regex::new(r"https?://\S+").unwrap();

    for mat in re.find_iter(input) {
        let start = mat.start();
        let url = mat.as_str();

        // Check what precedes the URL
        let preceded_by = if start > 0 {
            input.as_bytes()[start - 1]
        } else {
            b' '
        };

        result.push_str(&input[last_end..start]);

        if preceded_by == b'"' || preceded_by == b'\'' || preceded_by == b'=' {
            // Already inside an attribute — leave it alone
            result.push_str(url);
        } else {
            // Strip trailing punctuation that's likely not part of the URL
            let clean_url = url.trim_end_matches([')', '>', ']', ',', '.']);
            let trailing = &url[clean_url.len()..];
            result.push_str(&format!(
                r#"<a href="{}" style="color:#00aff4">{}</a>{}"#,
                clean_url, clean_url, trailing
            ));
        }

        last_end = mat.end();
    }
    result.push_str(&input[last_end..]);
    result
}

/// Escape HTML entities for safe display
fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bold() {
        assert!(parse_markdown("**hello**").contains("<strong>hello</strong>"));
    }

    #[test]
    fn test_italic() {
        assert!(parse_markdown("*hello*").contains("<em>hello</em>"));
    }

    #[test]
    fn test_bold_italic() {
        let result = parse_markdown("***hello***");
        assert!(result.contains("<strong><em>hello</em></strong>"));
    }

    #[test]
    fn test_underline() {
        assert!(parse_markdown("__hello__").contains("<u>hello</u>"));
    }

    #[test]
    fn test_strikethrough() {
        assert!(parse_markdown("~~hello~~").contains("<s>hello</s>"));
    }

    #[test]
    fn test_inline_code() {
        let result = parse_markdown("`code here`");
        assert!(result.contains("<code"));
        assert!(result.contains("code here"));
    }

    #[test]
    fn test_code_block() {
        let result = parse_markdown("```\ncode block\n```");
        assert!(result.contains("<pre"));
        assert!(result.contains("code block"));
    }

    #[test]
    fn test_spoiler() {
        let result = parse_markdown("||spoiler text||");
        assert!(result.contains("spoiler"));
        assert!(result.contains("spoiler text"));
    }

    #[test]
    fn test_user_mention() {
        let result = parse_markdown("<@123456>");
        assert!(result.contains("@user:123456"));
        assert!(result.contains("mention"));
    }

    #[test]
    fn test_channel_mention() {
        let result = parse_markdown("<#789>");
        assert!(result.contains("#channel:789"));
    }

    #[test]
    fn test_custom_emoji() {
        let result = parse_markdown("<:smile:123456>");
        assert!(result.contains("cdn.discordapp.com/emojis/123456.png"));
    }

    #[test]
    fn test_animated_emoji() {
        let result = parse_markdown("<a:dance:123456>");
        assert!(result.contains("cdn.discordapp.com/emojis/123456.gif"));
    }

    #[test]
    fn test_heading() {
        let result = parse_markdown("# Big Title");
        assert!(result.contains("24px"));
        assert!(result.contains("Big Title"));
    }

    #[test]
    fn test_masked_link() {
        let result = parse_markdown("[Click here](https://example.com)");
        assert!(result.contains("href=\"https://example.com\""));
        assert!(result.contains("Click here"));
    }

    #[test]
    fn test_block_quote() {
        let result = parse_markdown("> quoted text");
        assert!(result.contains("blockquote"));
        assert!(result.contains("quoted text"));
    }
}
