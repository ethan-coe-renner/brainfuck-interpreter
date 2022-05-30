use std::env;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::Read;

const MEMSIZE: usize = 30000;

struct State {
    instructions: Instructions,
    memory: [u8; MEMSIZE],
    pointer: usize,
}

#[derive(Debug)]
enum InterpreterError {
    UnmatchedBeginLoop(Vec<usize>),
    UnmatchedEndLoop(usize),
    NoInput,
}
impl Error for InterpreterError {}
impl fmt::Display for InterpreterError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::UnmatchedBeginLoop(locations) => {
                write!(
                    f,
                    "The '['s at these indices are unmatched: {:?}",
                    locations
                )
            }
            Self::UnmatchedEndLoop(location) => {
                write!(f, "Unmatched ']' at character {location}")
            }
            Self::NoInput => {
                write!(f, "No input given")
            }
        }
    }
}

impl State {
    fn initialize(instructions: Instructions) -> Self {
        Self {
            instructions,
            memory: [0; MEMSIZE],
            pointer: 0,
        }
    }

    fn update_state(&mut self) -> Result<(), InterpreterError> {
        // println!("Updating state");
        match self.instructions.instructions[self.instructions.pointer] {
            Instruction::IncPoint => {
                self.pointer += 1;
            }
            Instruction::DecPoint => self.pointer -= 1,
            Instruction::IncValue => self.memory[self.pointer] += 1,
            Instruction::DecValue => self.memory[self.pointer] -= 1,
            Instruction::LoopBegin => self.instructions.jump_stack.push(self.instructions.pointer),
            Instruction::LoopEnd => {
                match self.instructions.jump_stack.pop() {
                    Some(pointer) => {
                        if self.memory[self.pointer] != 0 {
                            self.instructions.pointer = pointer - 1; // subtract one because this fn adds one at end
                        }
                    }
                    None => {
                        return Err(InterpreterError::UnmatchedEndLoop(
                            self.instructions.pointer,
                        ))
                    }
                };
            }
            Instruction::GetChar => {
                let input: Option<u8> = std::io::stdin()
                    .bytes()
                    .next()
                    .and_then(|result| result.ok());
                match input {
                    Some(value) => self.memory[self.pointer] = value,
                    None => return Err(InterpreterError::NoInput),
                }
            }
            Instruction::PutChar => {
                print!("{}", self.memory[self.pointer] as char)
            }

            Instruction::Comment => {
                // do nothing on comments
            }
        }
        self.instructions.pointer += 1;
        Ok(())
    }

    fn run_program(&mut self) -> Result<(), InterpreterError> {
        while self.instructions.pointer < self.instructions.instructions.len() {
            self.update_state()?;
        }
        if self.instructions.jump_stack.is_empty() {
            Ok(())
        } else {
            Err(InterpreterError::UnmatchedBeginLoop(
                self.instructions.jump_stack.clone(),
            ))
        }
    }
}

#[derive(Debug)]
enum Instruction {
    IncPoint,
    DecPoint,
    IncValue,
    DecValue,
    LoopBegin,
    LoopEnd,
    GetChar,
    PutChar,
    Comment,
}

impl Instruction {
    fn new(instruction: u8) -> Self {
        match instruction {
            62 => Self::IncPoint,
            60 => Self::DecPoint,
            43 => Self::IncValue,
            45 => Self::DecValue,
            91 => Self::LoopBegin,
            93 => Self::LoopEnd,
            44 => Self::GetChar,
            46 => Self::PutChar,
            _ => Self::Comment,
        }
    }
}

struct Instructions {
    instructions: Vec<Instruction>,
    pointer: usize,
    jump_stack: Vec<usize>,
}

impl Instructions {
    fn new(instructions: Vec<Instruction>) -> Self {
        Instructions {
            instructions,
            pointer: 0,
            jump_stack: Vec::new(),
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    assert!(args.len() >= 1, "No file inputted");

    let mut state = State::initialize(get_instructions(&args[1])?);
    // println!("Instructions: {:?}", state.instructions.instructions);

    match state.run_program() {
        Err(error) => {
            println!("{error}")
        }
        Ok(()) => {
            println!("\nSuccessfully completed program")
        }
    }

    Ok(())
}

fn get_instructions(input_file: &str) -> Result<Instructions, Box<dyn Error>> {
    let mut input: Vec<u8> = Vec::new();
    File::open(input_file)?.read_to_end(&mut input)?;

    let instructions: Vec<Instruction> = input.iter().map(|x| Instruction::new(*x)).collect();

    Ok(Instructions::new(instructions))
}
