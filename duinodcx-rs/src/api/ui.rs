// duinodcx-rs/src/api/ui.rs
use axum::{http::header, response::IntoResponse};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "../dcx-ui-old/dist/"]
pub struct Asset;

pub async fn static_handler(uri: axum::http::Uri) -> axum::response::Response {
    let path = uri.path().trim_start_matches('/');
    if path.is_empty() || path == "index.html" { return index_handler().await; }
    match Asset::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
        }
        None => {
            if path.contains('.') { return (axum::http::StatusCode::NOT_FOUND, "404 Not Found").into_response(); }
            index_handler().await
        }
    }
}

pub async fn index_handler() -> axum::response::Response {
    match Asset::get("index.html") {
        Some(content) => {
            let html_str = String::from_utf8_lossy(&content.data);
            let injected_script = r#"
<script>
document.addEventListener('DOMContentLoaded', () => {
    const observer = new MutationObserver(() => {
        document.querySelectorAll('label').forEach(label => {
            if (label.innerText === 'Network' || label.innerText === 'Device / COM Port') {
                if (!label.dataset.refreshInjected) {
                    label.innerText = 'Device / COM Port';
                    label.dataset.refreshInjected = "true";
                    label.style.display = 'flex';
                    label.style.justifyContent = 'space-between';
                    label.style.alignItems = 'center';
                    const refreshBtn = document.createElement('button');
                    refreshBtn.innerHTML = '&#x21bb;'; // refresh icon
                    refreshBtn.style.border = 'none';
                    refreshBtn.style.background = 'none';
                    refreshBtn.style.color = '#007bff';
                    refreshBtn.style.cursor = 'pointer';
                    refreshBtn.onclick = (e) => {
                        e.preventDefault();
                        window.location.reload();
                    };
                    label.appendChild(refreshBtn);
                }
            }
            if (label.innerText === 'Password') {
                label.innerText = 'Baud Rate';
                const input = label.nextElementSibling;
                if (input && input.tagName === 'INPUT' && input.type === 'password') {
                    if (!input.dataset.baudInjected) {
                        input.dataset.baudInjected = "true";
                        input.style.display = 'none';
                        const select = document.createElement('select');
                        select.className = input.className;
                        ['9600','19200','38400','57600','115200'].forEach(b => {
                            const opt = document.createElement('option');
                            opt.value = b; opt.innerText = b;
                            if (b === '38400') opt.selected = true;
                            select.appendChild(opt);
                        });
                        const setNativeValue = (element, value) => {
                            const valueSetter = Object.getOwnPropertyDescriptor(element, 'value').set;
                            const prototype = Object.getPrototypeOf(element);
                            const prototypeValueSetter = Object.getOwnPropertyDescriptor(prototype, 'value').set;
                            if (valueSetter && valueSetter !== prototypeValueSetter) {
                                prototypeValueSetter.call(element, value);
                            } else {
                                valueSetter.call(element, value);
                            }
                        };
                        select.addEventListener('change', (e) => {
                            setNativeValue(input, e.target.value);
                            input.dispatchEvent(new Event('input', { bubbles: true }));
                        });
                        input.parentNode.insertBefore(select, input.nextSibling);
                        setNativeValue(input, '38400');
                        input.dispatchEvent(new Event('input', { bubbles: true }));
                    }
                }
            }
            if (label.innerText === 'IP') label.innerText = 'Connection Status';
        });

        document.querySelectorAll('button').forEach(btn => {
            if (btn.innerText === 'Connect') {
                btn.style.backgroundColor = '#dc3545';
                btn.style.borderColor = '#dc3545';
            } else if (btn.innerText === 'Disconnect') {
                btn.style.backgroundColor = '#28a745';
                btn.style.borderColor = '#28a745';
            }
        });
        
        document.querySelectorAll('input[disabled]').forEach(input => {
            if (input.value === 'Disconnected') {
                input.style.backgroundColor = '#dc3545';
                input.style.color = 'white';
            } else if (input.value.startsWith('Serial')) {
                input.style.backgroundColor = '#28a745';
                input.style.color = 'white';
            }
        });
    });
    observer.observe(document.body, { childList: true, subtree: true });
});
</script>
</body>
"#;
            let new_html = html_str.replace("</body>", injected_script);
            ([(header::CONTENT_TYPE, "text/html")], new_html).into_response()
        },
        None => (axum::http::StatusCode::NOT_FOUND, "404 Not Found").into_response(),
    }
}
