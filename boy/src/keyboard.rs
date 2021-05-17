use std::{char, collections::HashSet, cell::RefCell, rc::Rc};
use minifb::{Window, Key, KeyRepeat, InputCallback};

#[derive(Clone, Debug)]
pub struct Keyboard {
    keys:  Vec<bool>,
    codes: Rc<RefCell<HashSet<u32>>>,
    chars: Vec<char>,
}

impl Default for Keyboard {
    fn default() -> Self {
        let keys = vec![false; Key::Count as usize];
        let codes = Rc::new(RefCell::new(HashSet::new()));
        let chars = Vec::with_capacity(8);

        Keyboard { keys, codes, chars }
    }
}

impl Keyboard {
    pub fn new() -> Self {
        Keyboard::default()
    }

    pub fn update(&mut self, window: &Window) -> bool {
        let mut is_pressed = false;

        self.keys[Key::Escape as usize] = if window.is_key_down(Key::Escape) {
            is_pressed = true;
            true
        } else {
            false
        };

        self.keys[Key::LeftCtrl as usize] = window.is_key_down(Key::LeftCtrl);
        self.keys[Key::RightCtrl as usize] = window.is_key_down(Key::RightCtrl);

        for &key in &[
            Key::Home,
            Key::Left,
            Key::Right,
            Key::Up,
            Key::Down,
            Key::Space,
            Key::Key0,
            Key::Minus,
            Key::Equal,
        ] {
            self.keys[key as usize] = if window.is_key_pressed(key, KeyRepeat::Yes) {
                is_pressed = true;
                true
            } else {
                false
            };
        }

        is_pressed
    }

    #[inline]
    pub fn is_pressed(&self, key: Key) -> bool {
        self.keys[key as usize]
    }

    pub fn get_chars(&mut self) -> &[char] {
        self.chars.clear();
        self.chars.extend(self.codes.borrow_mut().drain().filter_map(char::from_u32));

        self.chars.as_slice()
    }
}

impl InputCallback for Keyboard {
    // FIXME use this callback for simulation control
    fn add_char(&mut self, char_code: u32) {
        // FIXME remove
        if char_code == 32 {
            let chars = self.get_chars();
            if !chars.is_empty() {
                println!("{:?}", chars);
            }
        } else {
            self.codes.borrow_mut().insert(char_code);
        }
    }
}
