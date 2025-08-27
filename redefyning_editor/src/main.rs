use redefyning::{
    init,
    WindowSettings,
};

fn main() {
    init(
        "TEST - Press ESC to Exit",
        (640, 480),
        Some(WindowSettings {
            resize: false,
            ..Default::default()
        }),
        None,
    );
}
