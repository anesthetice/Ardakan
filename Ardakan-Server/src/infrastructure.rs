use std::{
    time,
    thread,
    process::Command,
    os::windows::process::CommandExt,
};
use crate::constants::{
    SETVOL_PATH,
    SOUND_LIBRARY_PATH,
};
use windows::{
    s, core::*,
    Win32::Foundation::HWND,
    Win32::UI::WindowsAndMessaging::MessageBoxA,
};

fn sleep(time : &Option<f64>) {
    match time {
        Some(num) => {
            if *num > 0.0 {
                thread::sleep(time::Duration::from_secs_f64(*num));
            }
        },
        None => (),
    }
}
struct CMD {}
impl CMD {
    fn run(command : &str, creation_flag : u32) {
        Command::new("cmd").args(["/C", command]).creation_flags(creation_flag).spawn();
    }
    const NO_WINDOW : u32 = 0x08000000;
    const NEW_WINDOW : u32 = 0x00000010;
}

#[derive(Debug, Clone)]
pub enum InstructionType {
    Cmd,
    Sound,
    SoundLib,
    Echo,
    Website,
    SetVol,
    Sleep,
    Exit,
    Unknownn,
}

impl InstructionType {
    fn extract(string : &mut String) -> Self {
        if string.starts_with("cmd ") {
            *string = string.replace("cmd ", "");
            return Self::Cmd;
        }
        else if string.starts_with("bat ") {
            *string = string.replace("bat ", "");
            return Self::Cmd;
        }
        else if string.starts_with("sound ") {
            *string = string.replace("sound ", "");
            return Self::Sound;
        }
        else if string.starts_with("soundlib ") {
            *string = string.replace("soundlib ", "");
            return Self::SoundLib;
        }
        else if string.starts_with("echo ") {
            *string = string.replace("echo ", "");
            return Self::Echo;
        }
        else if string.starts_with("website ") {
            *string = string.replace("website ", "");
            return Self::Website
        }
        else if string.starts_with("setvol ") {
            *string = string.replace("setvol ", "");
            return Self::SetVol;
        }
        else if string.starts_with("sleep ") {
            *string = string.replace("sleep ", "");
            return Self::Sleep;
        }
        else if string.starts_with("exit") {
            *string = string.replace("exit", "");
            return Self::Exit;
        }
        else {
            return Self::Unknownn;
        }
    }
}

#[derive(Debug, Clone)]
enum Flag {
    Time(f64), // -t / --time
    Repeat(u64), // -r / --repeat
    Wait(f64), // -w / --wait
    Explicit, // -e / --explicit
}

impl Flag {
    fn extract(string : &mut String) -> Vec<Self> {
        let mut flags : Vec<Self> = Vec::new();
        let mut previous_element : &str = "";
        for element in (*string).clone().split(" ") {
            match previous_element {
                "-t" | "--time" => {
                    match element.parse::<f64>() {
                        Ok(num) => {
                            flags.push(Flag::Time(num));
                            *string = string.replace(&format!(" -t {}", element), "");
                            *string = string.replace(&format!(" --time {}", element), "");
                        },
                        Err(_) => (),
                    }
                },
                "-r" | "--repeat" => {
                    match element.parse::<u64>() {
                        Ok(num) => {
                            flags.push(Flag::Repeat(num));
                            *string = string.replace(&format!(" -r {}", element), "");
                            *string = string.replace(&format!(" --repeat {}", element), "");
                        },
                        Err(_) => (),
                    }
                },
                "-w" | "--wait" => {
                    match element.parse::<f64>() {
                        Ok(num) => {
                            flags.push(Flag::Wait(num));
                            *string = string.replace(&format!(" -w {}", element), "");
                            *string = string.replace(&format!(" --wait {}", element), "");
                        },
                        Err(_) => (),
                    }
                },
                "-e" | "--explicit" => {
                    flags.push(Flag::Explicit);
                    *string = string.replace(" -e", "");
                    *string = string.replace(" --explicit", "");
                },
                _ => (),
            }
            previous_element = element;
        }
        return flags;
    }
}
#[derive(Debug, Clone)]
pub struct Instruction {
    pub instruction_type_ : InstructionType,
    pub instruction_ : String,
    flags_ : Vec<Flag>,
}

impl Instruction {
    fn new(instruction_type : InstructionType, instruction : String, flags : Vec<Flag>) -> Self {
        return Self { instruction_type_:instruction_type, instruction_: instruction, flags_:flags };
    }

    pub fn from_string(mut string : String) -> Option<Self> {
        let flags : Vec<Flag> = Flag::extract(&mut string);
        let instruction_type : InstructionType = InstructionType::extract(&mut string);
        match instruction_type {
            InstructionType::Unknownn => return None,
            _ => return Some(Self::new(instruction_type, string, flags)),
        }
    }

    pub fn execute(&self) {
        let mut executions : u64 = 1;
        let mut time : Option<f64> = None;
        let mut wait : Option<f64> = None;
        let mut implicit : bool = true;

        for element in self.flags_.iter() {
            match *element {
                Flag::Time(num) => {time = Some(num)},                      
                Flag::Repeat(num) => {executions = num},
                Flag::Wait(num) => {wait = Some(num)},
                Flag::Explicit => {implicit=false},
            }
        }

        let instruction : String = self.instruction_.clone();

        match self.instruction_type_ {
            InstructionType::Cmd => {
                let creation_flag : u32 = match implicit {
                    true => CMD::NO_WINDOW,
                    false => CMD::NEW_WINDOW,
                };
                thread::spawn(move || {
                    let mut counter : u64 = 0;
                    while counter < executions {
                        CMD::run(&format!("{}", instruction), creation_flag);
                        sleep(&wait);
                        counter += 1;
                    }
                });
            },
            InstructionType::Sound => {
                match time {
                    Some(num) => {
                        thread::spawn(move || {
                            let mut counter : u64 = 0;
                            while counter < executions {
                                CMD::run(&format!("powershell -WindowStyle Hidden -Command (New-Object Media.SoundPlayer \'{}\').Play(); Start-Sleep -s {}; Exit;", instruction, num), CMD::NO_WINDOW);
                                sleep(&wait);
                                counter += 1;
                            }
                        });
                    },
                    None => {
                        thread::spawn(move || {
                            let mut counter : u64 = 0;
                            while counter < executions {
                                CMD::run(&format!("powershell -WindowStyle Hidden -Command (New-Object Media.SoundPlayer \'{}\').PlaySync();", instruction), CMD::NO_WINDOW);
                                sleep(&wait);
                                counter += 1;
                            }
                        });
                    },
                }
            },
            InstructionType::SoundLib => {
                match time {
                    Some(num) => {
                        thread::spawn(move || {
                            let mut counter : u64 = 0;
                            while counter < executions {
                                CMD::run(&format!("powershell -WindowStyle Hidden -Command (New-Object Media.SoundPlayer \'{}{}\').Play(); Start-Sleep -s {}; Exit;", SOUND_LIBRARY_PATH, instruction, num), CMD::NO_WINDOW);
                                sleep(&wait);
                                counter += 1;
                            }
                        });
                    },
                    None => {
                        thread::spawn(move || {
                            let mut counter : u64 = 0;
                            while counter < executions {
                                CMD::run(&format!("powershell -WindowStyle Hidden -Command (New-Object Media.SoundPlayer \'{}{}\').PlaySync();", SOUND_LIBRARY_PATH, instruction), CMD::NO_WINDOW);
                                sleep(&wait);
                                counter += 1;
                            }
                        });
                    },
                }
            },
            InstructionType::Echo => {
                thread::spawn(move || { 
                    let mut counter : u64 = 0;
                    while counter < executions {
                        unsafe {
                            match instruction.len() {
                                0 => {
                                    MessageBoxA(HWND(0),
                                    s!("default"),
                                    s!(""),
                                    Default::default());
                                },
                                _ => {
                                    let ptr : *const u8 = (*instruction).as_ptr();
                                    let pcstr : PCSTR = PCSTR::from_raw(ptr);
                                    MessageBoxA(HWND(0),
                                    pcstr,
                                    s!(""),
                                    Default::default());
                                },
                            };
                        };
                        sleep(&wait);
                        counter += 1;
                    }
                });
            },
            InstructionType::Website => {
                thread::spawn(move || {
                    let mut counter : u64 = 0;
                    while counter < executions {
                        CMD::run(&format!("rundll32 url.dll,FileProtocolHandler  {}", instruction), CMD::NO_WINDOW);
                        sleep(&wait);
                        counter += 1;
                    }
                });
            },
            InstructionType::SetVol => {
                thread::spawn(move || {
                    let mut counter : u64 = 0;
                    while counter < executions {
                        CMD::run(&format!("cd {} && setvol {}", SETVOL_PATH,  instruction), CMD::NO_WINDOW);
                        sleep(&wait);
                        counter += 1;
                    }
                });
            },
            InstructionType::Sleep => {
                let mut counter : u64 = 0;
                while counter < executions {
                    match instruction.trim().parse::<f64>() {
                        Ok(num) => {
                            if num > 0.0 {
                                thread::sleep(time::Duration::from_secs_f64(num))
                            }
                        },
                        Err(_) => break,
                    }
                    sleep(&wait);
                    counter += 1;
                }
            },
            InstructionType::Exit => {},
            InstructionType::Unknownn => {},
        }
    }
}

#[derive(Debug)]
struct RepeatMetaInstruction {
    number_of_repeats_ : u64,
    start_index_ : usize,
    stop_index_ : usize
}

impl RepeatMetaInstruction {
    fn new(number_of_repeats : u64, start_index : usize, stop_index : usize) -> Self {
        return Self { number_of_repeats_: number_of_repeats, start_index_: start_index, stop_index_: stop_index }
    }
}

#[derive(Debug)]
enum MetaInstruction {
    Repeat(RepeatMetaInstruction)
}

#[derive(Debug)]
pub struct InstructionsBuilder {
    instructions_precursor_ : Vec<Instruction>,
    meta_instructions_ : Vec<MetaInstruction>,
}

impl InstructionsBuilder {
    pub fn new() -> Self {
        let instructions_precursor : Vec<Instruction> = Vec::new();
        let meta_instructions : Vec<MetaInstruction> = Vec::new();
        return Self { instructions_precursor_: instructions_precursor, meta_instructions_: meta_instructions }
    }
    pub fn process(&mut self, mut combined_vector : Vec<&str>) {
        loop {
            let start_index = match combined_vector.iter().position(|s| s.starts_with("REPEAT ")) {
                Some(index) => index,
                None => break,
            };
            let number_of_repeats = match combined_vector.remove(start_index).replace("REPEAT ", "").parse::<u64>() {
                Ok(num) => num,
                Err(_) => continue,
            };
            let mut stop_index = match combined_vector.iter().rev().position(|s| s.starts_with(&format!("/REPEAT {}", &number_of_repeats))) {
                Some(index) => {
                    match combined_vector.iter().rev().position(|s| s.starts_with("/REPEAT ")) {
                        Some(check_index) => {
                            if check_index == index {
                                combined_vector.len() - (index + 1)
                            } else {
                                continue;
                            }
                        },
                        None => continue,
                    }
                },
                None => continue,
            }; combined_vector.remove(stop_index); if stop_index >= 1 {stop_index -= 1;};
            self.meta_instructions_.iter_mut().for_each(|meta| {
                match meta {
                    MetaInstruction::Repeat(repeat) => {
                        repeat.stop_index_ -= 2;
                    } 
                }
            });
            if stop_index >= start_index {
                self.meta_instructions_.push(MetaInstruction::Repeat(RepeatMetaInstruction::new(number_of_repeats, start_index, stop_index)));
            }
        }

        for element in combined_vector.into_iter() {
            match Instruction::from_string(element.to_owned()) {
                Some(instruction) => self.instructions_precursor_.push(instruction),
                None => {
                    self.meta_instructions_.iter_mut().for_each(|meta| {
                        match meta {
                            MetaInstruction::Repeat(repeat) => {
                                if repeat.stop_index_ >= 1 {repeat.stop_index_ -= 1;};
                            } 
                        }
                    });
                }
            }
        }
    }
    pub fn finalize(self) -> Vec<Instruction> {
        let instructions : Vec<Instruction> = Vec::new();

        if self.instructions_precursor_.len() == 0 {
            return instructions;
        }

        let mut super_instructions : Vec<Vec<Instruction>> = vec![instructions.clone(); self.instructions_precursor_.len()];
        super_instructions.iter_mut().enumerate().for_each(|(index, vector)| {
            vector.push(self.instructions_precursor_[index].clone())
        });

        for element in self.meta_instructions_.into_iter().rev() {
            match element {
                MetaInstruction::Repeat(repeat_instruction) => {
                    let mut super_instructions_clone = super_instructions.clone();
                    for index_pointer in (repeat_instruction.start_index_+1..=repeat_instruction.stop_index_).rev() {
                        super_instructions[repeat_instruction.start_index_].append(&mut super_instructions_clone[index_pointer]);
                        super_instructions[index_pointer].clear();
                    }
                    let mut _append_portion = super_instructions[repeat_instruction.start_index_].clone();
                    for _ in 1..repeat_instruction.number_of_repeats_ {
                        let mut append_portion = _append_portion.clone();
                        super_instructions[repeat_instruction.start_index_].append(&mut append_portion);
                    }
                }
            }
        }
        return super_instructions.into_iter().flatten().collect::<Vec<Instruction>>();
    }
}