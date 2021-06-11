use std::{
    char,
    collections::HashSet,
    cell::RefCell,
    rc::Rc,
    time::{Instant, Duration},
};
use winit::{
    event::{ElementState, KeyboardInput, ModifiersState, VirtualKeyCode},
};

#[derive(Clone, Default, Debug)]
pub struct KeyState {
    held_since: Option<Instant>,
    held_until: Option<Instant>,
    modifiers:  ModifiersState,
}

#[derive(Clone, Debug)]
pub struct Keyboard {
    modifiers:     ModifiersState,
    new_keys:      Vec<KeyState>,
    old_keys:      Vec<KeyState>,
    repeat_delay:  Duration,
    repeat_period: Duration,
    codes:         Rc<RefCell<HashSet<VirtualKeyCode>>>,
    chars:         Vec<char>,
}

impl Default for Keyboard {
    fn default() -> Self {
        let modifiers = ModifiersState::empty();
        let new_keys = vec![Default::default(); VirtualKeyCode::Cut as usize + 10];
        let old_keys = new_keys.clone();
        let repeat_delay = Duration::from_millis(500);
        let repeat_period = Duration::from_millis(100);
        let codes = Rc::new(RefCell::new(HashSet::new()));
        let chars = Vec::with_capacity(8);

        Keyboard { modifiers, new_keys, old_keys, repeat_delay, repeat_period, codes, chars }
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

    pub fn submit_key(&mut self, input: &KeyboardInput) {
        if let KeyboardInput { virtual_keycode: Some(keycode), state, .. } = input {
            let keycode = *keycode;
            match state {
                ElementState::Pressed => {
                    if keycode == VirtualKeyCode::Space {
                        // FIXME for testing only, remove
                        let chars = self.get_chars();
                        if !chars.is_empty() {
                            println!("{:?}", chars);
                        }
                    } else {
                        self.codes.borrow_mut().insert(keycode);
                    }

                    // FIXME grow (with sanity check)
                    if let Some(key_state) = self.new_keys.get_mut(keycode as usize) {
                        key_state.held_since = Some(Instant::now());
                        key_state.held_until = None;
                        key_state.modifiers = self.modifiers;
                    }
                }
                ElementState::Released => {
                    if let Some(key_state) = self.new_keys.get_mut(keycode as usize) {
                        key_state.held_until = Some(Instant::now());
                    }
                }
            }
        }
    }

    #[inline]
    pub fn submit_modifiers(&mut self, modifiers: &ModifiersState) {
        self.modifiers = *modifiers;
    }

    #[inline]
    pub fn is_pressed(&self, key: VirtualKeyCode) -> Option<ModifiersState> {
        if let Some(key_state) = self.new_keys.get(key as usize) {
            if key_state.held_since.is_some() && key_state.held_until.is_none() {
                Some(key_state.modifiers)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get_chars(&mut self) -> &[char] {
        self.chars.clear();
        self.chars
            .extend(self.codes.borrow_mut().drain().filter_map(|code| char::from_u32(code as u32)));
        self.chars.as_slice()
    }
}
