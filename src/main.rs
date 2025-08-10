#[path = "core/windowing/mod.rs"]
mod windowing;
use {
    std::{sync::{atomic::AtomicBool, mpsc}, thread},
    windowing::Windowing,
};

fn main() {
    println!("Welcome to the Redefyning Game Engine!");
    let (tx, rx) = mpsc::channel::<AtomicBool>();
    let mut windower = Windowing::new(tx);
    let thread_builder = thread::Builder::new().name("Renderer".to_string()).stack_size(8 * 1024 * 1024);
    let renderer = thread_builder.spawn(|| {println!("Hi")}).unwrap();
    renderer.join().unwrap();
    Windowing::run(&mut windower);
    // Initialize the game engine here
    // For example, set up the graphics, input handling, etc.
    // This is a placeholder for the actual game engine logic
    println!("Game engine initialized successfully.");
}
