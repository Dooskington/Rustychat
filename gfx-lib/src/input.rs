extern crate glutin;

use std::collections::{VecDeque, HashMap};
use glutin::{Event, WindowEvent, KeyboardInput, ElementState};
use ::*;

pub use glutin::VirtualKeyCode;

pub struct InputMan {
    current_keys: HashMap<VirtualKeyCode, bool>,
    pressed_keys: HashMap<VirtualKeyCode, bool>,
    released_keys: HashMap<VirtualKeyCode, bool>,
    pub input_string: String
}

impl InputMan {
    pub fn new() -> InputMan {
        InputMan {
            current_keys: HashMap::new(),
            pressed_keys: HashMap::new(),
            released_keys: HashMap::new(),
            input_string: String::new()
        }
    }

    pub fn clear_input_string(&mut self) {
        self.input_string = String::new();
    }
}

#[allow(dead_code)]
pub fn is_key_pressed(input_man: &InputMan, keycode: VirtualKeyCode) -> bool {
    *input_man.pressed_keys.get(&keycode).unwrap_or(&false)
}

#[allow(dead_code)]
pub fn is_key_released(input_man: &InputMan, keycode: VirtualKeyCode) -> bool {
    *input_man.released_keys.get(&keycode).unwrap_or(&false)
}

#[allow(dead_code)]
pub fn is_key_held(input_man: &InputMan, keycode: VirtualKeyCode) -> bool {
    *input_man.current_keys.get(&keycode).unwrap_or(&false)
}

pub fn process_events(window: &mut Window, input_man: &mut InputMan) {
    let mut events: VecDeque<Event> = VecDeque::new();
    window.events_loop.poll_events(|event| { events.push_back(event); });

    for event in events {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Closed => { window.is_close_requested = true; },
                WindowEvent::Resized(w, h) => { println!("Resize to {}, {}", w, h); resize_window(window, w, h); },
                WindowEvent::KeyboardInput { input, .. } => { input::process_key_input(input_man, &input); },
                _ => ()
            },
            _ => ()
        }
    }
}

pub fn update_input(input_man: &mut InputMan) {
    input_man.pressed_keys.clear();
    input_man.released_keys.clear();
}

fn process_key_input(input_man: &mut InputMan, event: &KeyboardInput) {
    let keycode: VirtualKeyCode = event.virtual_keycode.unwrap();

    match event.state {
        ElementState::Pressed => {
            if !input::is_key_held(input_man, keycode)
            {
                input_man.pressed_keys.insert(keycode, true);
            }

            input_man.current_keys.insert(keycode, true);

            if keycode == VirtualKeyCode::Back {
                input_man.input_string.pop();
            }

            // Add the key to the input string if possible
            if let Some(key_string) = keycode_to_string(keycode, event.modifiers.shift) {
                input_man.input_string.push_str(&key_string);
            }
        },
        ElementState::Released => {
            input_man.released_keys.insert(keycode, true);
            input_man.current_keys.insert(keycode, false);
        }
    }
}

fn keycode_to_string(keycode: VirtualKeyCode, is_upper: bool) -> Option<String> {
    let mut character: char;

    match keycode {
        VirtualKeyCode::A => character = 'a',
        VirtualKeyCode::B => character = 'b',
        VirtualKeyCode::C => character = 'c',
        VirtualKeyCode::D => character = 'd',
        VirtualKeyCode::E => character = 'e',
        VirtualKeyCode::F => character = 'f',
        VirtualKeyCode::G => character = 'g',
        VirtualKeyCode::H => character = 'h',
        VirtualKeyCode::I => character = 'i',
        VirtualKeyCode::J => character = 'j',
        VirtualKeyCode::K => character = 'k',
        VirtualKeyCode::L => character = 'l',
        VirtualKeyCode::M => character = 'm',
        VirtualKeyCode::N => character = 'n',
        VirtualKeyCode::O => character = 'o',
        VirtualKeyCode::P => character = 'p',
        VirtualKeyCode::Q => character = 'q',
        VirtualKeyCode::R => character = 'r',
        VirtualKeyCode::S => character = 's',
        VirtualKeyCode::T => character = 't',
        VirtualKeyCode::U => character = 'u',
        VirtualKeyCode::V => character = 'v',
        VirtualKeyCode::W => character = 'w',
        VirtualKeyCode::X => character = 'x',
        VirtualKeyCode::Y => character = 'y',
        VirtualKeyCode::Z => character = 'z',
        VirtualKeyCode::Space => character = ' ',
        VirtualKeyCode::Period => character = '.',
        VirtualKeyCode::Comma => character = ',',
        _ => return None
    }

    let mut string: String = character.to_string();
    if is_upper {
        string = string.to_uppercase();
    }

    Some(string)
}