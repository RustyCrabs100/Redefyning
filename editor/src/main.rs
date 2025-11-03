use redefyning::{App, AppVersion, WindowSettings};

fn script1() {
    println!("script 1 running!");
}

fn script2() {
    println!("script 2 running!");
}

fn main() {
    let app_version = AppVersion::new(0, 0, 0, 0, None);
    let app = App::new("test", app_version, None)
        .add_script(Box::new(script1))
        .add_script(Box::new(script2))
        .run();
}
