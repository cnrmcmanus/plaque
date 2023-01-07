use crate::engine::Exception;
use crate::instruction::Instruction;

pub const INCREMENT_POINTER: Instruction = Instruction {
    symbol: '>',

    exec: |program| {
        program.next_cell()?;
        program.next_instruction()
    },

    unexec: |program| {
        program.prev_cell()?;
        program.prev_instruction()
    },
};

pub const DECREMENT_POINTER: Instruction = Instruction {
    symbol: '<',

    exec: |program| {
        program.prev_cell()?;
        program.next_instruction()
    },

    unexec: |program| {
        program.next_cell()?;
        program.prev_instruction()
    },
};

pub const INCREMENT_CELL: Instruction = Instruction {
    symbol: '+',

    exec: |program| {
        program.map_cell(|cell| cell.wrapping_add(1));
        program.next_instruction()
    },

    unexec: |program| {
        program.map_cell(|cell| cell.wrapping_sub(1));
        program.prev_instruction()
    },
};

pub const DECREMENT_CELL: Instruction = Instruction {
    symbol: '-',

    exec: |program| {
        program.map_cell(|cell| cell.wrapping_sub(1));
        program.next_instruction()
    },

    unexec: |program| {
        program.map_cell(|cell| cell.wrapping_add(1));
        program.prev_instruction()
    },
};

pub const OUTPUT: Instruction = Instruction {
    symbol: '.',

    exec: |program| {
        program.output.push(program.cell());
        program.next_instruction()
    },

    unexec: |program| {
        program.output.pop();
        program.prev_instruction()
    },
};

pub const INPUT: Instruction = Instruction {
    symbol: ',',

    exec: |program| match program.pop_input() {
        None => Exception::RequestingInput.result(),
        Some(input) => {
            let cell = program.cell();
            program.set_cell(input);
            program.input_cell_history.push(cell);
            program.next_instruction()
        }
    },

    unexec: |program| match program.input_cell_history.pop() {
        None => Exception::error("no input to undo").result(),
        Some(cell) => {
            let input = program.cell();
            program.set_cell(cell);
            program.push_input(input);
            program.prev_instruction()
        }
    },
};

pub const JUMP_FORWARD: Instruction = Instruction {
    symbol: '[',

    exec: |program| {
        if program.cell() == 0 {
            program.goto_next(JUMP_BACKWARD)?;
        }
        program.next_instruction()
    },

    unexec: |program| match program.cell() {
        0 => program.goto_prev(JUMP_FORWARD),
        _ => program.prev_instruction(),
    },
};

pub const JUMP_BACKWARD: Instruction = Instruction {
    symbol: ']',

    exec: |program| {
        if program.cell() != 0 {
            program.goto_prev(JUMP_FORWARD)?;
        }
        program.next_instruction()
    },

    unexec: |program| match program.cell() {
        0 => program.prev_instruction(),
        _ => program.goto_next(JUMP_BACKWARD),
    },
};

pub const INSTRUCTION_SET: [Instruction; 8] = [
    INCREMENT_POINTER,
    DECREMENT_POINTER,
    INCREMENT_CELL,
    DECREMENT_CELL,
    OUTPUT,
    INPUT,
    JUMP_FORWARD,
    JUMP_BACKWARD,
];
