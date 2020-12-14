mod common;
mod api;
mod quake;

fn main() {
    env_logger::init();
    common::ArgParse::parse();
}