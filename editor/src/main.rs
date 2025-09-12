use redefyning::{WindowSettings, init};

fn main() {
    init(
        "TEST - Press ESC to Exit",
        (0, 0, 0, 0),
        (640, 480),
        Some(WindowSettings {
            resize: false,
            ..Default::default()
        }),
        None,
    );
}
