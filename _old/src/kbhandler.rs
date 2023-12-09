use {
    crate::*,
    pc_keyboard::{DecodedKey, KeyCode},
};

pub fn handle(key: DecodedKey) {
    if let DecodedKey::Unicode(key) = key {
        if key == '\n' {
            let screen = vga::SCREEN.lock();
            let idx = screen.idx;
            drop(screen);

            print!("\n");
            run_command(idx);

            return;
        }

        print!("{key}");
    }
}

fn run_command(command_end: usize) {
    let mut command_buffer = [0u8; vga::VgaTextBuffer::COL_MAX];
    let buffer = unsafe { &*vga::VgaTextBuffer::BUFFER };
    let command_len = (command_end % vga::VgaTextBuffer::COL_MAX) / 2;
    let command_end = command_end / 2;
    let command_start = command_end - command_len;

    let mut idx = 0;
    while idx < command_len {
        command_buffer[idx] = buffer[(idx * 2) + (command_start * 2)];
        idx += 1;
    }
    let command = core::str::from_utf8(&command_buffer[0..command_len])
        .unwrap()
        .trim();

    if command == "help" {
        println!(concat!(
            "Help:\n",
            "All commands are listed below. Arguments in `<>` are required. Arguments in `[]` are optional.\n",
            "echo [text] - Says [text] right back at you!\n",
            "add <num1> [num2]...[num n] - Adds all the numbers you give it together. Negatives and decimals are allowed.\n",
            "ping - Pong\n",
            "pong - Ping\n"
        ));
    } else if command.len() > 4 && &command[0..5] == "echo " {
        println!("{}", &command[5..]);
    } else if command == "echo" {
        println!();
    } else if command == "ping" {
        println!("pong");
    } else if command == "pong" {
        println!("ping");
    } else if command.len() > 5 && &command[0..3] == "add" {
        let mut nums = command[4..].split(' ');
        let mut running_total = 0f64;

        for num in nums {
            let Ok(num) = num.parse::<f64>() else {
                println!("Error: Invalid number `{num}`.");
                return;
            };
            running_total += num;
        }
        println!("{running_total}");
    } else if command == "add" || command == "add " {
        println!("Error: add needs at least 1 number to add.");
    } else {
        println!("Unknown command: `{command}`. Type `help` for help.");
    }
}
