pub const ROWS: usize = 12;
pub const COLS: usize = 36;

pub struct Terminal {
    pub buffer: [[u8; COLS]; ROWS],
    pub cursor_x: usize,
    pub cursor_y: usize,
    pub cmd_buffer: [u8; 64],
    pub cmd_len: usize,
}

impl Terminal {
    pub fn new() -> Self {
        let mut term = Terminal {
            buffer: [[b' '; COLS]; ROWS],
            cursor_x: 0,
            cursor_y: 0,
            cmd_buffer: [0; 64],
            cmd_len: 0,
        };
        term.write_str("Blog OS v0.1.0 (Plasma Terminal)\n");
        term.write_str("Type 'help' for commands.\n\n");
        term.print_prompt();
        term
    }

    pub fn print_prompt(&mut self) {
        self.write_str("$ ");
    }

    pub fn write_str(&mut self, s: &str) {
        for byte in s.bytes() {
            self.write_byte(byte);
        }
    }

    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => {
                self.cursor_y += 1;
                self.cursor_x = 0;
                if self.cursor_y >= ROWS {
                    self.scroll_up();
                }
            }
            c => {
                if self.cursor_x >= COLS {
                    self.cursor_y += 1;
                    self.cursor_x = 0;
                    if self.cursor_y >= ROWS {
                        self.scroll_up();
                    }
                }
                self.buffer[self.cursor_y][self.cursor_x] = c;
                self.cursor_x += 1;
            }
        }
    }

    fn scroll_up(&mut self) {
        for y in 1..ROWS {
            for x in 0..COLS {
                self.buffer[y - 1][x] = self.buffer[y][x];
            }
        }
        for x in 0..COLS {
            self.buffer[ROWS - 1][x] = b' ';
        }
        self.cursor_y = ROWS - 1;
    }

    pub fn process_input(&mut self, scancode: u8) {
        if (scancode & 0x80) != 0 { return; }

        if let Some(c) = crate::keyboard::scancode_to_ascii(scancode) {
            match c {
                '\n' => {
                    self.write_byte(b'\n');
                    let len = self.cmd_len;
                    let mut temp_cmd = [0u8; 64];
                    temp_cmd[..len].copy_from_slice(&self.cmd_buffer[..len]);
                    self.cmd_len = 0;

                    let cmd_str = core::str::from_utf8(&temp_cmd[..len]).unwrap_or("").trim();
                    self.execute_command(cmd_str);
                    self.print_prompt();
                }
                '\x08' => { // Backspace
                    if self.cmd_len > 0 {
                        self.cmd_len -= 1;
                        if self.cursor_x > 0 {
                            self.cursor_x -= 1;
                            self.buffer[self.cursor_y][self.cursor_x] = b' ';
                        }
                    }
                }
                ch => {
                    if self.cmd_len < self.cmd_buffer.len() - 1 {
                        let b = ch as u8;
                        self.cmd_buffer[self.cmd_len] = b;
                        self.cmd_len += 1;
                        self.write_byte(b);
                    }
                }
            }
        }
    }

    fn execute_command(&mut self, cmd: &str) {
        match cmd {
            "help" => {
                self.write_str("Commands:\n");
                self.write_str(" help    - show help\n");
                self.write_str(" clear   - clear screen\n");
                self.write_str(" version - OS version\n");
                self.write_str(" uname   - system info\n");
                self.write_str(" free    - memory info\n");
                self.write_str(" reboot  - reboot OS\n");
            }
            "clear" => {
                for y in 0..ROWS {
                    for x in 0..COLS {
                        self.buffer[y][x] = b' ';
                    }
                }
                self.cursor_x = 0;
                self.cursor_y = 0;
            }
            "version" => self.write_str("Blog OS v0.1.0 Plasma Edition\n"),
            "uname" => self.write_str("BlogOS x86_64 Kernel\n"),
            "free" => self.write_str("RAM: 640KB total, 128KB used\n"),
            "reboot" => {
                self.write_str("Rebooting...\n");
                crate::io::outb(0x64, 0xFE);
            }
            "" => {}
            _ => {
                self.write_str("Unknown: ");
                self.write_str(cmd);
                self.write_str("\n");
            }
        }
    }
}

