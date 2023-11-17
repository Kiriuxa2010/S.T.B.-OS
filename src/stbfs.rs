/* The infamous STBFS, its quite simple actually, its an in-memory filesystem. */

/* todo:
    remove unnecesary imports
    make the filesystem not in-memory
    make cd.. work 
*/
use crate::{print, println, vga_buffer::{print_shutdown}}; 
use alloc::string::String;
use lazy_static::lazy_static;
use spin::Mutex;
use crate::alloc::string::ToString;
use core::{
    pin::Pin,
    task::{Context, Poll},
};
use futures_util::{
    stream::{Stream, StreamExt},
    task::AtomicWaker,
};
use x86_64::instructions::hlt;
use alloc::{boxed::Box, vec, vec::Vec, rc::Rc};

// Define a file structure
#[derive(Clone)]
pub struct File {
    name: String,
    content: String,
}

// Define a directory structure
#[derive(Clone)]
pub struct Directory {
    name: String,
    files: Vec<File>,
    subdirectories: Vec<Directory>,
    parent: Option<usize>,
}

// Create the root directory
lazy_static! {
    static ref ROOT: Mutex<Directory> = Mutex::new(Directory {
        name: "$/".to_string(),
        files: vec![
            File {
                name: "file1.txt".to_string(),
                content: "This is file 1.".to_string(),
            },
            File {
                name: "file2.txt".to_string(),
                content: "This is file 2.".to_string(),
            },
        ],
        subdirectories: vec![
            Directory {
                name: "kernl".to_string(),
                files: vec![File {
                    name: "stbos.uff".to_string(),
                    content: "This is the Unreadable File Format, UFF for short".to_string(),
                }],
                subdirectories: vec![],
                parent: Some(0), // Set the parent to the index of the parent directory ("$/")
            },
        ],
        parent: None, // The root directory has no parent.
    });
}

// Function to list files in the current directory
pub fn ls() {
    let current_directory = ROOT.lock();
    println!("Contents of directory '{}':", current_directory.name);
    for file in &current_directory.files {
        println!("File: {}", file.name);
    }
    for subdir in &current_directory.subdirectories {
        println!("Directory: {}", subdir.name);
    }
}

pub fn cat(filename: &str) {
    let current_directory = ROOT.lock();
    if let Some(file) = current_directory.files.iter().find(|f| f.name == filename) {
        println!("\n{}", file.content);
    } else {
        println!("\nFile '{}' not found.", filename);
    }
}

// Function to change the current directory
pub fn cd(new_directory: &str) {
    let mut current_directory = ROOT.lock();

    if new_directory == ".." {
        if let Some(parent_index) = current_directory.parent {
            // Print some debug information
            println!("Changing to parent directory (index: {})", parent_index);
            println!("Current directory name: {}", current_directory.name);
            println!("Parent directory name: {}", current_directory.subdirectories[parent_index].name);
        
            // Change the current directory to the parent directory
            current_directory.parent = current_directory.subdirectories[parent_index].parent;
            *current_directory = current_directory.subdirectories[parent_index].clone();
        
            println!("New current directory name: {}", current_directory.name);
        } else {
            println!("\nAlready at the root directory.");
        }
         
    } else {
        if let Some(sub_index) = current_directory.subdirectories.iter().position(|dir| dir.name == new_directory) {
            // Print some debug information
            println!("Changing to subdirectory (index: {})", sub_index);

            // Change the current directory to the specified subdirectory
            current_directory.parent = Some(sub_index);
            *current_directory = current_directory.subdirectories[sub_index].clone();
        } else {
            println!("\nDirectory '{}' not found.", new_directory);
        }
    }
}

pub fn mkdir(new_directory: &str) {
    let mut current_directory = ROOT.lock();

    // Check if the new directory name is empty
    if new_directory.is_empty() {
        println!("\nDirectory name cannot be empty.");
        return;
    }

    // Check if the new directory name already exists
    if current_directory.subdirectories.iter().any(|dir| dir.name == new_directory) {
        println!("\nDirectory '{}' already exists.", new_directory);
        return;
    }

    // Create the new directory and add it to the current directory
    current_directory.subdirectories.push(Directory {
        name: new_directory.to_string(),
        files: vec![],
        subdirectories: vec![],
        parent: Some(0), // Set the parent to the current directory
    });

    println!("\nDirectory '{}' created.", new_directory);
}

pub fn touch(filename: &str, content: &str) {
    let mut current_directory = ROOT.lock();

    // Check if the new file name is empty
    if filename.is_empty() {
        println!("\nFile name cannot be empty.");
        return;
    }

    // Check if the new file name already exists
    if current_directory.files.iter().any(|file| file.name == filename) {
        println!("\nFile '{}' already exists.", filename);
        return;
    }

    // Create the new file and add it to the current directory
    current_directory.files.push(File {
        name: filename.to_string(),
        content: content.to_string(),
    });

    println!("\nFile '{}' created.", filename);
}