# simple_6502rs
## [Web App](https://zahamza.github.io/simple_6502rs/)
Web App deployed on link above. This app is made more fun and learning, so code will probably break if its pushed too hard (I added some guards, however).

/docs and its scripts were slightly modfied version of the [egui template](https://github.com/emilk/egui_template/). If you want to compile the wasm on your own or use the scripts, [read here](https://github.com/emilk/egui_template/blob/master/README.md#compiling-for-the-web), it follows the same structure.

## Instructions
Using the emulator should be pretty intuitive if you know how a 6502 works.  **Note: Decimal Mode is not supported currently**

Inputs will be set to 0 if you don't eneter the input in hex or properly. To load a program to a specific address in RAM, first specify the hex address in  the "Start Address" field. Then copy and paste the **object code**, or assembly, that you wish to load in the box under the "Start Address" field. This **object code** can work with or without whitespace. The only requirement is that the all the non-whitespace characters can be classified as Hexadecimal (both capital and lowercase letters will work).

If you click *Continous Run* the emulator will run until a **BRK** opcode is reached (or an unexpected panic happens!).  

*CPU Reset* will reset the internal registers of the CPU.

*Clear* will reset both the internal registers and the RAM.

## Compiling Locally
You will need to have Rust and Cargo installed [(see here)](https://www.rust-lang.org/tools/install)

To compile the native version, simply run "cargo run --release" while in the equivalent of the /simple_6502rs directory.

You can read the [egui template](https://github.com/emilk/egui_template/blob/master/README.md) for more detailed instruction on how to compile for the web. The gist of it is, while in /simple_6502rs directory first run the ./setup_web.sh script. After this one can build with ./build_web.sh and start a server with ./start_server.sh. If you decide to change directory paths you may need to modify these scripts


## Credits
I used the [egui library](https://github.com/emilk/egui) to code the application/GUI portion and to compile the app to wasm. 

## Potential Additions
* I need to refactor some code, especially in app.rs
* Program in interrupts into the GUI (cpu code already present)
* Display total cycles ran (cpu code present, needs to be placed in GUI)
* Display more disassembled instructions (potentially in window format)
* Clean up GUI so it fits better with full screen
* Implement decimal mode

## Purpose (Ramblings)
I wanted to build a NES emulator and I also wanted to learn Rust. I then decided to code a NES emulator, and the cpu code found here is the first actual coding of that effort. I wanted to also try to use a GUI for the fun of it so I decided on egui because it was easy to use and could compile as both a native and web app. This was made for fun and to learn, so if anyone actually reads the code please remember that 😅. 

This app may also be used to help me debug my CPU and more easily test things while developing my NES emulator further. I will use this code to help finish my own NES emulation, if you're interested in making your own check out some of the resources I used or I thought could be helpful (I didn't make any of these).


## Resources 
* Masswerk [Instruction Set](https://www.masswerk.at/6502/6502_instruction_set.html)
* javidx9's [NES CPU Video](https://www.youtube.com/watch?v=8XmxKPJDGU0)
* All of the masswerk tools were useful, heres a link to the [assembler](https://www.masswerk.at/6502/assembler.html) and [emulator](https://www.masswerk.at/6502/index.html)
* Obelisk [Instruction Reference](http://obelisk.me.uk/6502/reference.html)
* To understand carry flag and how it was can be used as a borrow [(link)](https://en.wikipedia.org/wiki/Carry_flag#Vs._borrow_flag)
* I used this to reaffrim some of my assembly understanding [Easy 6502](https://skilldrick.github.io/easy6502/)
* NESDEV wiki: read a bit for a lot but [specifically](http://wiki.nesdev.com/w/index.php/Status_flags#The_B_flag)
*  [@bugzmanov](http://twitter.com/bugzmanov)'s ["Writing a NES Emulator in Rust"](https://bugzmanov.github.io/nes_ebook/chapter_1.html) gave me some ideas for how to write certain things in Rust

