use std::{env, error::Error, fs::File, io::Read};
use thiserror::Error;

const MEMSIZE: usize = 30000; // brainfuck spec defines a 30000 byte memory

// An error type to represent errors encountered during interpretation
#[derive(Error, Debug)]
enum InterpreterError {
    #[error("Unmatched ']'")]
    UnmatchedBeginLoop(Vec<usize>),
    #[error("Unmatched ']'")]
    UnmatchedEndLoop(usize),
    #[error("Memory pointer incremented above MEMSIZE={MEMSIZE}")]
    MemPointerBelowBounds,
    #[error("Memory pointer decremented below 0")]
    MemPointerAboveBounds,
    #[error("No input given")]
    NoInput,
}

// struct representing current state machine, including memory, memory pointer, and instructions
struct State {
    instructions: Instructions,
    memory: [u8; MEMSIZE],
    mem_pointer: usize,
}

impl State {
    // Initialize state based on spec
    fn initialize(instructions: Instructions) -> Self {
        Self {
            instructions,
            memory: [0; MEMSIZE],
            mem_pointer: 0,
        }
    }

    // update state by interpreting current instruction
    fn update_state(&mut self) -> Result<(), InterpreterError> {
        match self.instructions.instructions[self.instructions.pointer] {
            Instruction::IncPoint => {
                if self.mem_pointer < MEMSIZE - 1 {
                    self.mem_pointer += 1;
                } else {
                    return Err(InterpreterError::MemPointerAboveBounds);
                }
            }
            Instruction::DecPoint => {
                if self.mem_pointer > 0 {
                    self.mem_pointer -= 1;
                } else {
                    return Err(InterpreterError::MemPointerBelowBounds);
                }
            }
            Instruction::IncValue => {
                self.memory[self.mem_pointer] = self.memory[self.mem_pointer].wrapping_add(1)
            }
            Instruction::DecValue => {
                self.memory[self.mem_pointer] = self.memory[self.mem_pointer].wrapping_sub(1)
            }
            Instruction::LoopBegin => self.instructions.jump_stack.push(self.instructions.pointer),
            Instruction::LoopEnd => {
                match self.instructions.jump_stack.pop() {
                    Some(pointer) => {
                        if self.memory[self.mem_pointer] != 0 {
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
                    Some(value) => self.memory[self.mem_pointer] = value,
                    None => return Err(InterpreterError::NoInput),
                }
            }
            Instruction::PutChar => {
                print!("{}", self.memory[self.mem_pointer] as char)
            }

            Instruction::Comment => {
                // do nothing on comments
            }
        }
        self.instructions.pointer += 1; // increment instruction pointer
        Ok(())
    }

    // Update state until EOF
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

// Enumeration defining Instructions in BF
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
    // map ASCII to BF instructions
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

// Instructions struct represents a set of instructions
// includes a vector of instructions, a pointer to the current instruction, and a stack of jumps (for loops)
struct Instructions {
    instructions: Vec<Instruction>,
    pointer: usize,
    jump_stack: Vec<usize>,
}

impl Instructions {
    // Initializes instructions based on spec
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
    assert!(args.len() > 1, "No file inputted");

    let mut state = State::initialize(get_instructions(&args[1])?);

    match state.run_program() {
        Err(error) => {
            println!("\nInterpreter Error: {error}")
        }
        Ok(()) => {
            println!("\nSuccessfully completed program")
        }
    }

    Ok(())
}

// read instructions from file
fn get_instructions(input_file: &str) -> Result<Instructions, Box<dyn Error>> {
    let mut input: Vec<u8> = Vec::new();
    File::open(input_file)?.read_to_end(&mut input)?;

    let instructions: Vec<Instruction> = input.iter().map(|x| Instruction::new(*x)).collect();

    Ok(Instructions::new(instructions))
}
