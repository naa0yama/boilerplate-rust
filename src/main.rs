pub mod libs;
use crate::libs::hello::sayhello;

fn main() {
    let name = sayhello(String::from("naa0yama"));
    println!("{}, new world!!", name);
}
