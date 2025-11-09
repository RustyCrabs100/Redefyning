use crate::utils::AppState;
use crate::vk::setup;

pub struct Core(setup::VulkanSetup);

impl Core {
    pub fn new(input: setup::VulkanSetup) -> Self {
        Core(input)
    }

    pub fn main_loop(self) {
        'main: loop {
            let receiver = match &self.0.window_communicator {
                Some(x) => x,
                None => panic!("Unable to receive orders! Shutting down."),
            };

            match receiver.recv().unwrap() {
                AppState::Awaiting => {
                    println!("Awaiting orders...");
                }
                AppState::Closed => {
                    println!("Closing...");
                    break 'main;
                }
                _ => {}
            }
        }
    }
}
