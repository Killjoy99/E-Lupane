//! Small shared helpers for the dashboard UI (client + server).

use crate::assignments_types;

/// Format an optional UTC timestamp for display.
pub fn fmt_dt(dt: Option<chrono::DateTime<chrono::Utc>>) -> String {
    match dt {
        Some(d) => d.format("%Y-%m-%d %H:%M").to_string(),
        None => "—".to_string(),
    }
}

/// Read the first file of `<input type="file" id=...>` as base64.
pub async fn read_file_base64(input_id: &str) -> Option<assignments_types::FileUploadInput> {
    let js = format!(
        r#"(() => {{
            const el = document.getElementById('{input_id}');
            const f = el && el.files && el.files[0];
            if (!f) return 'null';
            return new Promise((resolve) => {{
                const r = new FileReader();
                r.onload = () => resolve(JSON.stringify({{ name: f.name, b64: r.result.split(',')[1] }}));
                r.readAsDataURL(f);
            }});
        }})()"#
    );
    let eval = dioxus::document::eval(&js);
    let out: String = eval.join().await.unwrap_or_default();
    let v: serde_json::Value = serde_json::from_str(&out).ok()?;
    Some(assignments_types::FileUploadInput {
        name: v
            .get("name")
            .and_then(|n| n.as_str())
            .unwrap_or("file")
            .to_string(),
        mime_type: "application/octet-stream".to_string(),
        content_base64: v
            .get("b64")
            .and_then(|b| b.as_str())
            .unwrap_or("")
            .to_string(),
    })
}
