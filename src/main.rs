#[macro_use]
extern crate quicli;
use quicli::prelude::*;
use std::io;
use std::io::Read;
use std::process::exit;

#[derive(Debug, StructOpt)]
struct Cli {
    /// minimum cell value
    #[structopt(short = "a", default_value = "0")]
    minimum: usize,

    /// maximum cell value
    #[structopt(short = "b", default_value = "255")]
    maximum: usize,

    /// number of cells to allocate
    #[structopt(short = "c", default_value = "30000")]
    cells: usize,

    /** value to store upon EOF, which can be one of:
    0    store a zero in the cell
    a    store the minimum cell value in the cell
    b    store the maximum cell value in the cell
    n    store a negative one in the cell
    x    do not change the cell's contents
    */
    #[structopt(short = "e", default_value = "0")]
    eof_value: String,

    /// The file to read from, or - for stdin
    file: String,

    /** runtime mode, which can be one of:
    d    dump parsed code
    r    run normally
    */
    #[structopt(short = "m", default_value = "r")]
    runtime: String,

    /// value overflow/underflow behaviour
    #[structopt(short = "l", default_value = "w")]
    value_behaviour: String,

    /** cell pointer overflow/underflow behaviour

    overflow/underflow behaviours can be one of:
    e    throw an error and quit upon over/underflow
    i    do nothing when attempting to over/underflow
    w    wrap around to the other end upon over/underflow
    */
    #[structopt(short = "p", default_value = "w")]
    pointer_behaviour: String,

    #[structopt(flatten)]
    verbosity: Verbosity,
}

#[derive(Debug, PartialEq)]
enum InstructionType {
    EndError,
    EndIgnore,
    EndWrap,
    OpVinc,
    OpVdec,
    OpPinc,
    OpPdec,
    OpLstart,
    OpLend,
    OpIn,
    OpOut,
    EofZero,
    EofMin,
    EofMax,
    EofNegone,
    EofNochg,
}

#[derive(Debug)]
struct Instruction {
    /// instruction type
    instruction: InstructionType,
    /// number of times to run the instruction
    quantity: usize,
    /// index of the loop's matching other instruction
    loopi: usize,
}

fn preprocess(instructions: &mut Vec<Instruction>, buffer: &String) {
    let mut index = 0;
    for c in buffer.chars() {
        match c {
            '+' => instructions.push(Instruction {
                instruction: InstructionType::OpVinc,
                quantity: 1,
                loopi: 0,
            }),
            '-' => instructions.push(Instruction {
                instruction: InstructionType::OpVdec,
                quantity: 1,
                loopi: 0,
            }),
            '>' => instructions.push(Instruction {
                instruction: InstructionType::OpPinc,
                quantity: 1,
                loopi: 0,
            }),
            '<' => instructions.push(Instruction {
                instruction: InstructionType::OpPdec,
                quantity: 1,
                loopi: 0,
            }),
            '\n' => continue,
            _ => exit(1),
        }
        if index > 0 {
            // group nearby together
            if instructions[index - 1].instruction == instructions[index].instruction {
                instructions.pop(); // remove the newest
                instructions[index - 1].quantity += 1;
            } else {
                index += 1;
            }
        } else {
            index += 1;
        }
    }
}

fn execute(
    instructions: &Vec<Instruction>,
    cells: &mut Vec<u8>,
    cell_index: usize,
    instruction_index: usize,
) {
    if instruction_index > instructions.len() - 1 {
        return;
    }
    let instruction = &instructions[instruction_index];
    let mut next_iindex = instruction_index + 1;
    let mut next_cindex = cell_index;
    match instruction.instruction {
        InstructionType::OpVinc => cells[cell_index] += instruction.quantity as u8,
        InstructionType::OpVdec => cells[cell_index] -= instruction.quantity as u8,
        InstructionType::OpPinc => next_cindex += instruction.quantity,
        InstructionType::OpPdec => next_cindex -= instruction.quantity,
        _ => (),
    }

    return execute(instructions, cells, next_cindex, next_iindex);
}

main!(|args: Cli, log_level: verbosity| {
    let mut buffer = String::new();

    if args.file == "-" {
        // Read from stdin
        io::stdin().read_to_string(&mut buffer)?;
    } else {
        buffer = read_file(args.file)?;
    }

    let mut instructions: Vec<Instruction> = Vec::new();
    preprocess(&mut instructions, &buffer);
    // println!("{:?}", instructions);
    let mut cells: Vec<u8> = vec![0; args.cells];
    execute(&mut instructions, &mut cells, 0, 0);

    println!("finished!");
    println!("{:?}", cells);
});
