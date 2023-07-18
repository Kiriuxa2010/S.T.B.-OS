// #[no_std]
// extern crate volatile;

// use crate::println;
// use crate::print;

// pub fn main() {
//     // Initialize the serial port
//     let mut serial_port = unsafe { volatile::Volatile::new(0x3F8) };

//     // Print the command prompt
//     serial_port.write(b"> ");

//     // Read input from the serial port
//     let mut command = String::new();
//     while true {
//         let byte = unsafe { serial_port.read() };
//         if byte == b'\r' {
//             // Run the command
//             let output = Command::new("sh")
//                 .arg("-c")
//                 .arg(command.trim())
//                 .output()
//                 .unwrap_or_else(|e| {
//                     println!("Failed to execute command: {}", e);
//                     return;
//                 });

//             // Print the output to the serial port
//             serial_port.write(&output.stdout);
//             serial_port.write(&output.stderr);

//             // Clear the command and print the prompt again
//             command.clear();
//             serial_port.write(b"> ");
//         } else {
//             command.push(byte as char);
//         }
//     }
// }
