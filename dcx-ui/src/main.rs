#![allow(non_snake_case)]

mod api;
mod components;

use dioxus::prelude::*;
use components::connection::ConnectionPanel;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        div {
            style: "font-family: sans-serif; padding: 20px;",
            h1 { "WinRS-DCX Dioxus UI" }
            ConnectionPanel {}
        }
    }
}
