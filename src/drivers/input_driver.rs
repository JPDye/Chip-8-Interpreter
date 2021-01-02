use sdl2::{self, event::Event, keyboard::Keycode};

pub struct InputDriver {
    events: sdl2::EventPump,
}

impl InputDriver {
    pub fn new(sdl_context: &sdl2::Sdl) -> Self {
        InputDriver {
            events: sdl_context.event_pump().unwrap(),
        }
    }

    pub fn poll(&mut self) -> Result<Option<u8>, ()> {
        for event in self.events.poll_iter() {
            if let Event::Quit { .. } = event {
                return Err(());
            }
        }

        let keys: Vec<Keycode> = self
            .events
            .keyboard_state()
            .pressed_scancodes()
            .filter_map(Keycode::from_scancode)
            .collect();

        // Map key from modern keyboard to hexadecimal Chip8 keypad.
        for key in keys {
            match key {
                Keycode::Num1 => return Ok(Some(0x1)),
                Keycode::Num2 => return Ok(Some(0x2)),
                Keycode::Num3 => return Ok(Some(0x3)),
                Keycode::Num4 => return Ok(Some(0xC)),

                Keycode::Q => return Ok(Some(0x4)),
                Keycode::W => return Ok(Some(0x5)),
                Keycode::E => return Ok(Some(0x6)),
                Keycode::R => return Ok(Some(0xD)),

                Keycode::A => return Ok(Some(0x7)),
                Keycode::S => return Ok(Some(0x8)),
                Keycode::D => return Ok(Some(0x9)),
                Keycode::F => return Ok(Some(0xE)),

                Keycode::Z => return Ok(Some(0xA)),
                Keycode::X => return Ok(Some(0x0)),
                Keycode::C => return Ok(Some(0xB)),
                Keycode::V => return Ok(Some(0xF)),

                Keycode::Space => return Ok(Some(0xFF)),

                _ => (),
            }
        }
        Ok(None)
    }
}
