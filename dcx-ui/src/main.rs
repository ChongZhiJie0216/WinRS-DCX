#![allow(non_snake_case)]

mod api;

use dioxus::prelude::*;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        div {
            h1 { "WinRS-DCX Controller" }
        }
    }
}
