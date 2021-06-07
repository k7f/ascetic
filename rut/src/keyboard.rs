use std::{char, collections::HashMap, time::{Instant, Duration}};
use winit::{
    event::{ElementState, Event, KeyboardInput, ModifiersState, VirtualKeyCode, WindowEvent},
};

#[derive(Clone, Default, Debug)]
pub struct KeyState {
    held_since: Option<Instant>, // FIXME HashMap<ModifiersState, Instant>
    modifiers: ModifiersState,
}

#[derive(Clone, Debug)]
pub struct Keyboard {
    modifiers: ModifiersState,
    keys: Vec<KeyState>,
    repeat_delay: Duration,
    repeat_period: Duration,
}

impl Default for Keyboard {
    fn default() -> Self {
        let modifiers = ModifiersState::empty();
        let keys = vec![Default::default(); VirtualKeyCode::Cut as usize + 10];
        let repeat_delay = Duration::from_millis(500);
        let repeat_period = Duration::from_millis(100);

        Keyboard { modifiers, keys, repeat_delay, repeat_period }
    }
}

impl Keyboard {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn set_repeat_delay(&mut self, duration: Duration) {
        self.repeat_delay = duration;
    }

    pub fn set_repeat_period(&mut self, duration: Duration) {
        self.repeat_period = duration;
    }

    pub fn add_key(&mut self, input: &KeyboardInput) {
        if let KeyboardInput {
            virtual_keycode: Some(keycode),
            state,
            ..
        } = input {
            // FIXME grow (with sanity check)
            if let Some(key_state) = self.keys.get_mut(*keycode as usize) {
                match state {
                    ElementState::Pressed => {
                        key_state.held_since = Some(Instant::now());
                        key_state.modifiers = self.modifiers.clone();
                    }
                    ElementState::Released => {
                        key_state.held_since = None;
                    }
                }
            }
        }
    }

    #[inline]
    pub fn set_modifiers(&mut self, modifiers: &ModifiersState) {
        self.modifiers = modifiers.clone();
    }

    #[inline]
    pub fn is_pressed(&self, key: VirtualKeyCode) -> Option<ModifiersState> {
        if let Some(key_state) = self.keys.get(key as usize) {
            if key_state.held_since.is_some() {
                Some(key_state.modifiers.clone())
            } else {
                None
            }
        } else {
            None
        }
    }
}

// impl InputCallback for Keyboard {
//     // FIXME use this callback for simulation control
//     fn add_char(&mut self, char_code: u32) {
//         // FIXME remove
//         if char_code == 32 {
//             let chars = self.get_chars();
//             if !chars.is_empty() {
//                 println!("{:?}", chars);
//             }
//         } else {
//             self.codes.borrow_mut().insert(char_code);
//         }
//     }
// }
