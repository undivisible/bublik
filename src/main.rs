#![allow(dead_code)]

mod app;
mod audio;
mod state;
mod ui;

fn main() {
    leptos::mount::mount_to_body(app::App);
}
