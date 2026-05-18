use dioxus::prelude::*;
use crate::api::{fetch_ports, fetch_connection, connect_port, disconnect_port};

#[component]
pub fn ConnectionPanel() -> Element {
    let mut ports = use_signal(|| Vec::<String>::new());
    let mut selected_port = use_signal(|| String::new());
    let mut status = use_signal(|| "Disconnected".to_string());
    let mut is_connected = use_signal(|| false);

    // Initial load
    use_resource(move || async move {
        if let Ok(p) = fetch_ports().await {
            ports.set(p.clone());
            if !p.is_empty() {
                selected_port.set(p[0].clone());
            }
        }
        if let Ok(c) = fetch_connection().await {
            if !c.current.is_empty() {
                selected_port.set(c.current);
                status.set(c.ip);
                is_connected.set(true);
            }
        }
    });

    let on_connect = move |_| {
        spawn(async move {
            if is_connected() {
                if disconnect_port().await.is_ok() {
                    status.set("Disconnected".to_string());
                    is_connected.set(false);
                }
            } else {
                let port = selected_port();
                if connect_port(&port).await.is_ok() {
                    if let Ok(c) = fetch_connection().await {
                        status.set(c.ip);
                        is_connected.set(true);
                    }
                }
            }
        });
    };

    rsx! {
        div {
            style: "padding: 10px; border: 1px solid #ccc; margin-bottom: 20px;",
            h2 { "Connection" }
            div {
                style: "display: flex; gap: 10px; align-items: center;",
                label { "Device / COM Port:" }
                select {
                    value: "{selected_port}",
                    onchange: move |evt| selected_port.set(evt.value().clone()),
                    disabled: is_connected(),
                    for port in ports() {
                        option { value: "{port}", "{port}" }
                    }
                }
                button {
                    onclick: on_connect,
                    style: if is_connected() { "background-color: #28a745; color: white;" } else { "background-color: #dc3545; color: white;" },
                    if is_connected() { "Disconnect" } else { "Connect" }
                }
            }
            div {
                style: "margin-top: 10px;",
                span { "Status: {status}" }
            }
        }
    }
}
