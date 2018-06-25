#[macro_use]
extern crate quicli;
use quicli::prelude::*;
use std::io;
use std::io::Read;

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

enum Instruction {
    OpVinc { quantity: usize },
    OpVdec { quantity: usize },
    OpPinc { quantity: usize },
    OpPdec { quantity: usize },
    OpIn { quantity: usize },
    OpOut { quantity: usize },
    OpLstart { destination: usize },
    OpLend { destination: usize },
}

impl Instruction {
    fn inc(&mut self) {
        match self {
            Instruction::OpVinc { quantity }
            | Instruction::OpVdec { quantity }
            | Instruction::OpPinc { quantity }
            | Instruction::OpPdec { quantity }
            | Instruction::OpIn { quantity }
            | Instruction::OpOut { quantity } => {
                *quantity += 1;
            }
            _ => (),
        }
    }
}

// https://stackoverflow.com/a/32554326/4376737
fn variant_eq<T>(a: &T, b: &T) -> bool {
    std::mem::discriminant(a) == std::mem::discriminant(b)
}

fn preprocess(instructions: &mut Vec<Instruction>, buffer: &String) {
    let mut index = 0;
    let mut stack: Vec<usize> = Vec::new();
    for c in buffer.chars() {
        match c {
            // FIXME: Wrap
            '+' => instructions.push(Instruction::OpVinc { quantity: 1 }),
            '-' => instructions.push(Instruction::OpVdec { quantity: 1 }),
            '>' => instructions.push(Instruction::OpPinc { quantity: 1 }),
            '<' => instructions.push(Instruction::OpPdec { quantity: 1 }),
            '.' => instructions.push(Instruction::OpOut { quantity: 1 }),
            '[' => {
                stack.push(instructions.len());
                instructions.push(Instruction::OpLstart { destination: 0 });
            }
            ']' => {
                let dest = match stack.pop() {
                    Some(dest) => dest,
                    _ => panic!("Unmatched ] operator"),
                };
                instructions[dest] = Instruction::OpLstart {
                    destination: instructions.len(),
                };
                instructions.push(Instruction::OpLend { destination: dest + 1 });
            }
            _ => continue, // comments or newline
        }
        if index > 0 {
            // group nearby together
            if variant_eq(&instructions[index - 1], &instructions[index]) {
                instructions.pop(); // remove the newest
                instructions[index - 1].inc();
            } else {
                index += 1;
            }
        } else {
            index += 1;
        }
    }
    if stack.len() > 0 {
        panic!("Not enough ] operators!");
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
    match instruction {
        Instruction::OpVinc { quantity } => cells[cell_index] += *quantity as u8,
        Instruction::OpVdec { quantity } => cells[cell_index] -= *quantity as u8,
        Instruction::OpPinc { quantity } => next_cindex += *quantity,
        Instruction::OpPdec { quantity } => next_cindex -= *quantity,
        Instruction::OpOut { quantity } => for _ in 0..*quantity {
            print!("{}", cells[cell_index] as char)
        },
        Instruction::OpLstart { destination } => if cells[cell_index] == 0 {
            next_iindex = *destination;
        },
        Instruction::OpLend { destination } => if cells[cell_index] > 0 {
            next_iindex = *destination;
        },
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
});
