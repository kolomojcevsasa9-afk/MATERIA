use crate::vga_buffer;
use crate::keyboard;
use crate::io;
use crate::sfs;

pub fn init() {
    vga_buffer::clear_screen();
    vga_buffer::print_colored(vga_buffer::Color::LightGreen, format_args!("========================================\n"));
    vga_buffer::print_colored(vga_buffer::Color::LightCyan,  format_args!("   Welcome to Blog OS (SFS File System) \n"));
    vga_buffer::print_colored(vga_buffer::Color::LightGreen, format_args!("========================================\n"));
    vga_buffer::print_colored(vga_buffer::Color::Yellow,     format_args!("Type 'help' for a list of commands.\n\n"));
    print_prompt();
}

fn print_prompt() {
    vga_buffer::print_colored(vga_buffer::Color::LightBlue, format_args!("blog_os"));
    vga_buffer::print_colored(vga_buffer::Color::DarkGray,  format_args!(":"));
    vga_buffer::print_colored(vga_buffer::Color::LightGreen,format_args!("~"));
    vga_buffer::print_colored(vga_buffer::Color::White,     format_args!("$ "));
}

pub fn run() {
    let mut cmd_buffer = [0u8; 64];
    let mut cmd_len = 0;

    loop {
        let scancode = io::read_scancode();
        if scancode != 0 && (scancode & 0x80) == 0 {
            while io::read_scancode() == scancode {}

            if let Some(c) = keyboard::scancode_to_ascii(scancode) {
                if c == '\n' {
                    crate::println!();
                    process_command(&cmd_buffer[..cmd_len]);
                    cmd_len = 0;
                    print_prompt();
                } else if c == '\x08' {
                    if cmd_len > 0 {
                        cmd_len -= 1;
                        vga_buffer::backspace();
                    }
                } else {
                    if cmd_len < cmd_buffer.len() - 1 {
                        cmd_buffer[cmd_len] = c as u8;
                        cmd_len += 1;
                        crate::print!("{}", c);
                    }
                }
            }
        }
    }
}

fn process_command(cmd_bytes: &[u8]) {
    let cmd = core::str::from_utf8(cmd_bytes).unwrap_or("").trim();
    match cmd {
        "help" => {
            crate::println!("BusyBox-style commands:");
            crate::println!("  help                 - Display this help message");
            crate::println!("  clear                - Clear the screen");
            crate::println!("  version              - Display OS version");
            crate::println!("  about                - About Blog OS");
            crate::println!("  echo [text]          - Echo text");
            crate::println!("  ls                   - List SFS files");
            crate::println!("  touch [file]         - Create an SFS file");
            crate::println!("  mkdir [dir]          - Create a directory");
            crate::println!("  cat [file]           - Read SFS file");
            crate::println!("  rm [file]            - Delete SFS file");
            crate::println!("  write [file] [text]  - Write text to SFS file");
            crate::println!("  pwd                  - Print working directory");
            crate::println!("  uname                - System information");
            crate::println!("  free                 - Memory information");
            crate::println!("  reboot               - Reboot system");
            crate::println!("  nvme                 - Initialize NVMe");
        }
        "clear" => vga_buffer::clear_screen(),
        "version" => crate::println!("Blog OS v0.1.0"),
        "about" => crate::println!("A custom hobby operating system."),
        "ls" => sfs::ls(),
        "pwd" => crate::println!("/"),
        "uname" => crate::println!("BlogOS i686 x86_PC"),
        "free" => {
            crate::println!("total        used        free");
            crate::println!("640 KB      128 KB      512 KB");
        }
        "reboot" => {
            crate::println!("Rebooting...");
            io::outb(0x64, 0xFE);
        }
        "nvme" => {
            let mut found = false;
            for bus in 0..8 {
                for slot in 0..32 {
                    let vendor_id = (crate::pci::read_config_u32(bus, slot, 0, 0) & 0xFFFF) as u16;
                    let class_code = (crate::pci::read_config_u32(bus, slot, 0, 0x08) >> 24) as u8;
                    let subclass = ((crate::pci::read_config_u32(bus, slot, 0, 0x08) >> 16) & 0xFF) as u8;

                    if class_code == 0x01 && subclass == 0x08 {
                        crate::println!("Found NVMe: VID {:#x}", vendor_id);
                        let nvme = crate::nvme::NvmeDriver::new(bus, slot);
                        crate::sfs::init(nvme);
                        crate::println!("NVMe initialized and SFS attached.");
                        found = true;
                        break;
                    }
                }
            }
            if !found { crate::println!("No NVMe device found."); }
        }
        s if s.starts_with("echo ") => crate::println!("{}", &s[5..]),
        s if s.starts_with("touch ") => {
            let filename = s[6..].trim();
            if !filename.is_empty() && sfs::touch(filename) {
                crate::println!("Created file '{}'.", filename);
            }
        }
        s if s.starts_with("mkdir ") => {
            let dirname = s[6..].trim();
            if !dirname.is_empty() && sfs::mkdir(dirname) {
                crate::println!("Created directory '{}'.", dirname);
            }
        }
        s if s.starts_with("cat ") => sfs::cat(s[4..].trim()),
        s if s.starts_with("rm ") => {
            let filename = s[3..].trim();
            if !filename.is_empty() && sfs::rm(filename) {
                crate::println!("Removed '{}'.", filename);
            }
        }
        s if s.starts_with("write ") => {
            let args = s[6..].trim();
            let mut parts = args.splitn(2, ' ');
            if let (Some(filename), Some(content)) = (parts.next(), parts.next()) {
                if sfs::write_file(filename, content) {
                    crate::println!("Wrote to '{}'.", filename);
                }
            }
        }
        "" => {}
        other => crate::println!("Unknown command: '{}'.", other),
    }
}
