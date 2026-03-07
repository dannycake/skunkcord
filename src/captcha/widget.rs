// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! hCaptcha widget HTML generation
//!
//! Generates a self-contained HTML page that embeds the hCaptcha widget
//! with all Enterprise parameters (sitekey, rqdata). The widget communicates
//! the solution back via document.title changes.

use super::CaptchaChallenge;

/// Generate the HTML page for the hCaptcha widget
///
/// This page:
/// 1. Loads hCaptcha API with explicit rendering mode
/// 2. Renders the widget with the provided sitekey
/// 3. Calls hcaptcha.setData() with rqdata blob (if present) — CRITICAL for Enterprise
/// 4. Reports solution/error/expiry via document.title changes
pub fn generate_captcha_html(challenge: &CaptchaChallenge) -> String {
    let sitekey = escape_js_string(&challenge.captcha_sitekey);
    let rqdata_script = if let Some(ref rqdata) = challenge.captcha_rqdata {
        format!(
            "hcaptcha.setData(widgetId, {{ rqdata: '{}' }});",
            escape_js_string(rqdata)
        )
    } else {
        String::new()
    };

    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>Captcha Challenge</title>
    <style>
        body {{
            margin: 0;
            padding: 0;
            display: flex;
            justify-content: center;
            align-items: center;
            min-height: 100vh;
            background-color: #313338;
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
            color: #dbdee1;
        }}
        .container {{
            text-align: center;
            padding: 24px;
        }}
        .title {{
            font-size: 20px;
            font-weight: 600;
            margin-bottom: 16px;
            color: #ffffff;
        }}
        .subtitle {{
            font-size: 14px;
            color: #949ba4;
            margin-bottom: 24px;
        }}
        #captcha-container {{
            display: inline-block;
            min-height: 78px;
        }}
        .loading {{
            color: #949ba4;
            font-size: 14px;
        }}
        .error {{
            color: #f23f43;
            font-size: 14px;
            margin-top: 16px;
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="title">Verification Required</div>
        <div class="subtitle">Please complete the captcha to continue</div>
        <div id="captcha-container">
            <div class="loading">Loading captcha...</div>
        </div>
        <div id="error-msg" class="error" style="display:none;"></div>
    </div>

    <script src="https://js.hcaptcha.com/1/api.js?onload=onLoadCaptcha&render=explicit" async defer></script>
    <script>
        function onLoadCaptcha() {{
            var container = document.getElementById('captcha-container');
            container.innerHTML = '';

            try {{
                var widgetId = hcaptcha.render('captcha-container', {{
                    sitekey: '{sitekey}',
                    theme: 'dark',
                    size: 'normal',
                    callback: onSolved,
                    'error-callback': onError,
                    'expired-callback': onExpired,
                    'chalexpired-callback': onExpired
                }});

                // CRITICAL: Forward Enterprise rqdata blob
                // Without this, the challenge appears to work but the solved token
                // will be rejected by Discord's backend
                {rqdata_script}

            }} catch (e) {{
                showError('Failed to initialize captcha: ' + e.message);
                document.title = 'CAPTCHA_ERROR:init_failed';
            }}
        }}

        function onSolved(token) {{
            // Pass solved token back to the Rust application via title change
            document.title = 'CAPTCHA_SOLVED:' + token;
        }}

        function onError(err) {{
            showError('Captcha error: ' + (err || 'unknown'));
            document.title = 'CAPTCHA_ERROR:' + (err || 'unknown');
        }}

        function onExpired() {{
            showError('Captcha expired. Please try again.');
            document.title = 'CAPTCHA_EXPIRED';
        }}

        function showError(msg) {{
            var el = document.getElementById('error-msg');
            el.textContent = msg;
            el.style.display = 'block';
        }}
    </script>
</body>
</html>"#,
        sitekey = sitekey,
        rqdata_script = rqdata_script,
    )
}

/// Escape a string for safe inclusion in JavaScript string literals
fn escape_js_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('<', "\\x3c")
        .replace('>', "\\x3e")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_widget_html_includes_sitekey() {
        let challenge = CaptchaChallenge {
            captcha_service: "hcaptcha".to_string(),
            captcha_sitekey: "test-sitekey-123".to_string(),
            captcha_rqdata: None,
            captcha_rqtoken: None,
            captcha_session_id: None,
            captcha_key: None,
        };
        let html = generate_captcha_html(&challenge);
        assert!(html.contains("sitekey: 'test-sitekey-123'"));
    }

    #[test]
    fn test_widget_html_includes_rqdata_when_present() {
        let challenge = CaptchaChallenge {
            captcha_service: "hcaptcha".to_string(),
            captcha_sitekey: "test-key".to_string(),
            captcha_rqdata: Some("blob123data==".to_string()),
            captcha_rqtoken: None,
            captcha_session_id: None,
            captcha_key: None,
        };
        let html = generate_captcha_html(&challenge);
        assert!(html.contains("hcaptcha.setData(widgetId"));
        assert!(html.contains("rqdata: 'blob123data=='"));
    }

    #[test]
    fn test_widget_html_omits_rqdata_when_absent() {
        let challenge = CaptchaChallenge {
            captcha_service: "hcaptcha".to_string(),
            captcha_sitekey: "test-key".to_string(),
            captcha_rqdata: None,
            captcha_rqtoken: None,
            captcha_session_id: None,
            captcha_key: None,
        };
        let html = generate_captcha_html(&challenge);
        assert!(!html.contains("setData"));
    }

    #[test]
    fn test_widget_html_escapes_special_chars() {
        let challenge = CaptchaChallenge {
            captcha_service: "hcaptcha".to_string(),
            captcha_sitekey: "key'with\"special<chars>".to_string(),
            captcha_rqdata: Some("data'with\"quotes".to_string()),
            captcha_rqtoken: None,
            captcha_session_id: None,
            captcha_key: None,
        };
        let html = generate_captcha_html(&challenge);
        // Should not contain unescaped quotes or angle brackets
        assert!(!html.contains("key'with"));
        assert!(html.contains("key\\'with"));
        assert!(!html.contains("<chars>"));
    }

    #[test]
    fn test_widget_html_has_all_callbacks() {
        let challenge = CaptchaChallenge {
            captcha_service: "hcaptcha".to_string(),
            captcha_sitekey: "test".to_string(),
            captcha_rqdata: None,
            captcha_rqtoken: None,
            captcha_session_id: None,
            captcha_key: None,
        };
        let html = generate_captcha_html(&challenge);
        assert!(html.contains("function onSolved(token)"));
        assert!(html.contains("function onError(err)"));
        assert!(html.contains("function onExpired()"));
        assert!(html.contains("CAPTCHA_SOLVED:"));
        assert!(html.contains("CAPTCHA_ERROR:"));
        assert!(html.contains("CAPTCHA_EXPIRED"));
    }

    #[test]
    fn test_widget_html_loads_hcaptcha_script() {
        let challenge = CaptchaChallenge {
            captcha_service: "hcaptcha".to_string(),
            captcha_sitekey: "test".to_string(),
            captcha_rqdata: None,
            captcha_rqtoken: None,
            captcha_session_id: None,
            captcha_key: None,
        };
        let html = generate_captcha_html(&challenge);
        assert!(html.contains("js.hcaptcha.com/1/api.js"));
        assert!(html.contains("render=explicit"));
        assert!(html.contains("onload=onLoadCaptcha"));
    }

    #[test]
    fn test_js_escape() {
        assert_eq!(escape_js_string("hello"), "hello");
        assert_eq!(escape_js_string("it's"), "it\\'s");
        assert_eq!(escape_js_string("<script>"), "\\x3cscript\\x3e");
        assert_eq!(escape_js_string("a\"b"), "a\\\"b");
    }
}
