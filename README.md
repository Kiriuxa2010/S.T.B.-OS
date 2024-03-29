# S.T.B.™ OS (Simple Terminal Based Operating System)

S.T.B.™ OS is a 64-bit operating system written in Rust, inspired by classic operating systems like MS-DOS. This project aims to provide a simple, lightweight, and terminal-based operating system.

<img width="720" alt="image" src="https://github.com/Kiriuxa2010/S.T.B.-OS/assets/71524929/16030955-3143-4c2c-9c77-644f1628d2a5">


## Table of Contents
- [Features](#features)
- [Getting Started](#getting-started)
- [Usage](#usage)
- [Contributing](#contributing)
- [License](#license)

## Latest Change Logs
Version 0.9.8.6: 
* added mkdir command.
* added touch command.
* changed up some colors
  
Current Bugs:
* /cd .. causes a kernel panic
* backspace key kinda works(minor bugs)

## Features
S.T.B.™ OS offers several features to help you understand its capabilities:

* **Terminal Interface**: S.T.B.™ OS provides a command-line interface for interacting with the system, just like classic operating systems.

* **Multiple Colors**: Enjoy colorful text and graphics in your terminal.

* **Keyboard Support**: Interact with the OS using your keyboard.

* **Double Fault System**: Improved stability with a double fault mechanism.

* **Async/Await**: Take advantage of asynchronous programming for efficient multitasking.

* **Heap Allocation**: Dynamically allocate memory for your programs.

* **Working Commands**: A set of working commands to perform various tasks.

*  **Custom Filesystem**: A custom filesystem specifically made for S.T.B.™ OS

> Note: The project is a work in progress, and further enhancements are planned.

## Getting Started
To get started with S.T.B.™ OS, follow these steps:

1. **Prerequisites**: Ensure you have Rust installed on your system. You can download Rust from [rust-lang.org](https://www.rust-lang.org/).
   **rust must be nightly!**
   Install These too:
   ```shell
    rustup component add llvm-tools-preview
    rustup target add thumbv7em-none-eabihf
    cargo build --target thumbv7em-none-eabihf

    rustup target add admiralix_os.json
    cargo build --target admiralix_os.json
    ```
3. **Clone the Repository**:
   ```shell
   git clone https://github.com/Kiriuxa2010/S.T.B.-OS.git
   cd S.T.B.-OS
4. **Build It**:
   ```shell
   cargo bootimage
the compiled .bin file will be in the folder S.T.B.-OS/target/admiralix_os/debug

5. **Running It**:
   to run S.T.B.™ OS in qemu you can use this command:
   ```shell
   qemu-system-x86_64 -drive format=raw,file=bootimage-admiralix_os.bin
   ```
   You can also burn the .bin file to a usb if you want to

> Note: **If you dont want to build it, you can get a ready .bin file here:**
https://drive.google.com/drive/folders/1Cq6whB1-5AxlTZ5ChoEjYwC44aXyW9Di?usp=sharing

## Usage
Once you have S.T.B.™ OS running, you can use the terminal interface to execute commands and explore the OS. The available commands and their usage will be documented in the project as it evolves.
Use the command /syshelp to show a list of currently possible commands.


## Contributing
Contributing
We welcome contributions to the project. If you want to help improve S.T.B.™ OS, please follow these steps:

Fork the repository.

Create a new branch for your feature or bug fix:
```shell
git checkout -b feature/my-feature
````
Make your changes and commit them:
```shell
git commit -m "Add a new feature"
```
Push your changes to your fork:
```shell
git push origin feature/my-feature
```
Open a pull request on this repository, detailing your changes and improvements.

## License
This project is licensed under the MIT License - see the LICENSE file for details.
