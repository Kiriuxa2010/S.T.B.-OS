// #![no_std]
// #![no_main]

// use core::str;
// use core::fmt::Write;
// use core::panic::PanicInfo;
// use uart_16550::SerialPort;
// // Panic handler
// #[panic_handler]
// fn panic(_info: &PanicInfo) -> ! {
//     loop {}
// }

// // Entry point
// #[no_mangle]
// pub fn init_shell() {
//     let mut serial_port = unsafe { SerialPort::new(0x3f8) };
//     let mut input_buffer = [0u8; 1024];
//     let mut input_len = 0usize;

//     // Display prompt
//     serial_port.write(b"> ");

//     loop {
//         // Read input
//         let byte = serial_port.read_byte();
//         match byte {
//             // Backspace/Delete
//             0x08 | 0x7f => {
//                 if input_len > 0 {
//                     input_len -= 1;
//                     input_buffer[input_len] = 0;
//                     serial_port.write(&[0x08, b' ', 0x08]);
//                 }
//             },
//             // Newline/Carriage return
//             0x0a | 0x0d => {
//                 serial_port.write(&[b'\r', b'\n']);
//                 if input_len > 0 {
//                     let input_str = str::from_utf8(&input_buffer[..input_len]).unwrap();
//                     serial_port.write(input_str.as_bytes());
//                     serial_port.write(&[b'\r', b'\n']);
//                     input_len = 0;
//                     input_buffer.fill(0);
//                 }
//                 serial_port.write(b"> ");
//             },
//             // Other characters
//             byte if byte.is_ascii() => {
//                 if input_len < input_buffer.len() {
//                     input_buffer[input_len] = byte;
//                     input_len += 1;
//                     serial_port.write(&[byte]);
//                 }
//             },
//             _ => (),
//         }
//     }
// }
