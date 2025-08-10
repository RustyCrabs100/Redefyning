// This file is used for unpacking the windowing module
// Example: pub use window::window; 
// In the above example, we unpack the window module from the window file, as to prevent the user from having to type window::window every time they want to use the window module.
// This should be done for all modules belonging to windowing, as to make it easier for the user to access them.
mod base;
pub use base::windowing::Windowing;