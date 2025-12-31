use rand::{rngs::ThreadRng, Rng};
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[derive(Debug)]
pub struct Roll {
    // The number of dice to be rolled
    pub num_rolls: usize,
    // The sides of each die
    pub dice_sides: u8,
    // i.e. is this being added or subtracted from our running total?
    pub multiplier: i8,
    // Do we drop the highest, lowest, or no dice
    // (i.e. for advantage/disadvantage in DnD)
    pub drop_die: Option<DropDie>,
    // The results of each individual roll
    pub results: Vec<u32>
}

impl Roll {
    pub fn new() -> Self {
        Roll { num_rolls: 0, dice_sides: 0, multiplier: 1, drop_die: None, results: Vec::new() }
    }
}

// The states of our FSM, used to consume the user input
pub enum States {
    ObtainingNumberOfDice,
    ObtainingDiceSides,
    ObtainingDropDieType,
    ObtainingDropDieNum,
    ObtainingNextOperation
}

#[derive(Debug)]
pub enum DropDie {
    DropLowest(usize),
    DropHighest(usize)
}


// Add to a running total and build up an output string
pub fn add_to_total_and_output(index: usize, roll: &mut Roll, total: &mut i64, output: &mut String, rng: &mut ThreadRng, skip_dropped: bool, colour: bool) -> Result<(), Box<dyn std::error::Error>> {
    // Roll the dice
    if roll.num_rolls == 0 {
        return Ok(());
    }
    roll.results.reserve(roll.num_rolls);
    for _ in 0..roll.num_rolls {
        roll.results.push(rng.random_range(1..=roll.dice_sides).into())
    }
    if index > 0 {
        match roll.multiplier {
            1 => {
                *output += " + ";
            }
            -1 => {
                *output += " - ";
            }
            _ => {
                return Err("Multiplier somehow obtained as neither 1 or -1".into());
            }
        }
    }
    if roll.num_rolls == 1 {
        *output += &roll.results.first().unwrap().to_string();
    }
    // If we are dropping, then...
    match &roll.drop_die {
        Some(y) => {
            let drop_n: usize;
            match y {
                DropDie::DropLowest(x) => {
                    if *x >= roll.num_rolls {
                        return Ok(());
                    }
                    roll.results.sort_by_key(|n| *n);
                    drop_n = *x;
                    
                }
                DropDie::DropHighest(x) => {
                    if *x >= roll.num_rolls {
                        return Ok(());
                    }
                    roll.results.sort_by_key(|n| *n);
                    roll.results.reverse();
                    drop_n = *x;
                }
            }
            // If skipping,
            if skip_dropped {
                // Start by removing the dropped rolls
                roll.results.drain(0..drop_n);
                // And then add to output string
                if roll.results.len() > 1 {
                    output.push('('); 
                    for (result_index, result) in roll.results.iter().enumerate() {
                        if result_index > 0 {
                            *output += " + "
                        }
                        *output += &result.to_string();
                    }
                    output.push(')');
                }
                else {
                    for (result_index, result) in roll.results.iter().enumerate() {
                        if result_index > 0 {
                            *output += " + "
                        }
                        *output += &result.to_string();
                    }
                }
            }
            // If not skipping,
            else {
                // Start with adding to output string
                if roll.num_rolls > 1 && !skip_dropped {
                    output.push('('); 
                    for (result_index, result) in roll.results.iter().enumerate() {
                        if result_index > 0 {
                            *output += " + "
                        }
                        if result_index < drop_n {
                            if colour {
                                *output += "\x1b[0;91mX\x1b[0m";
                            }
                            else {
                                *output += "X";
                            }
                        }
                        *output += &result.to_string();
                    }
                    output.push(')');
                }
                // And end with removing the dropped dice
                roll.results.drain(0..drop_n);
            }
        }
        None => {
            if roll.num_rolls > 1 {
                output.push('('); 
                for (result_index, result) in roll.results.iter().enumerate() {
                    if result_index > 0 {
                        *output += " + "
                    }
                    *output += &result.to_string();
                }
                output.push(')');
            }
        }
    }
    *total += ((roll.multiplier as i32) * (roll.results.iter().sum::<u32>() as i32)) as i64;
    Ok(())
}

// Only add to the running total, doing no string manipulation
pub fn add_to_total(roll: &mut Roll, total: &mut i64, rng: &mut ThreadRng) -> Result<(), Box<dyn std::error::Error>> {
    // Roll the dice
    if roll.num_rolls == 0 {
        return Ok(());
    }
    roll.results.reserve(roll.num_rolls);
    for _ in 0..roll.num_rolls {
        roll.results.push(rng.random_range(1..=roll.dice_sides).into())
    }
    // If we are dropping, then...
    match &roll.drop_die {
        Some(y) => {
            let drop_n: usize;
            match y {
                DropDie::DropLowest(x) => {
                    if *x >= roll.num_rolls {
                        return Ok(());
                    }
                    roll.results.sort_by_key(|n| *n);
                    drop_n = *x;
                    
                }
                DropDie::DropHighest(x) => {
                    if *x >= roll.num_rolls {
                        return Ok(());
                    }
                    roll.results.sort_by_key(|n| *n);
                    roll.results.reverse();
                    drop_n = *x;
                }
            }
            // And end with removing the dropped dice
            roll.results.drain(0..drop_n);
        }
        None => {}
    }
    *total += ((roll.multiplier as i32) * (roll.results.iter().sum::<u32>() as i32)) as i64;
    Ok(())
}

pub fn consume_input_to_rolls(input: &String, help_message: &String) -> Result<Vec<Roll>, Box<dyn std::error::Error>> {
    let mut rolls: Vec<Roll> = Vec::new();
    let mut new_roll = Roll::new();
    let mut state: States = States::ObtainingNumberOfDice;
    for character in input.trim().chars() {
        // Depending on our current state,
        match state {
            States::ObtainingDiceSides => {
                if character.is_digit(10) {
                    new_roll.dice_sides *= 10;
                    new_roll.dice_sides += character.to_digit(10).ok_or("Expected character to be a digit")? as u8;
                }
                else {
                    match character {
                        ' ' => {
                            state = States::ObtainingNextOperation;
                            rolls.push(new_roll);
                            new_roll = Roll::new();
                        }
                        '+' => {
                            state = States::ObtainingNumberOfDice;
                            rolls.push(new_roll);
                            new_roll = Roll::new();
                        }
                        '-' => {
                            new_roll.multiplier = -1;
                            state = States::ObtainingNumberOfDice;
                            rolls.push(new_roll);
                            new_roll = Roll::new();
                        }
                        'd' => {
                            state = States::ObtainingDropDieType;
                        }
                        _ => {
                            println!("{}", help_message);
                            return Err("Unknown character encountered".into());
                        }
                    }
                }
            }

            States::ObtainingNumberOfDice => {
                if character.is_digit(10) {
                    new_roll.num_rolls *= 10;
                    new_roll.num_rolls += character.to_digit(10).ok_or("Expected character to be a digit")? as usize;
                }
                else {
                    match character {
                        'd' => {
                            state = States::ObtainingDiceSides;
                            if new_roll.num_rolls == 0 {
                                new_roll.num_rolls = 1;
                            }
                        }
                        ' ' => {
                            continue;
                        }
                        _ => {
                            println!("{}", help_message);
                            return Err("Unknown character encountered".into());
                        }
                    }
                }
            }

            States::ObtainingDropDieType => {
                match character {
                    'l' => {
                        new_roll.drop_die = Some(DropDie::DropLowest(0));
                        state = States::ObtainingDropDieNum;
                    }
                    'h' => {
                        new_roll.drop_die = Some(DropDie::DropHighest(0));
                        state = States::ObtainingDropDieNum;
                    }
                    _ => {
                        println!("{}", help_message);
                        return Err("Unknown character encountered".into());
                    }
                }
            }
            States::ObtainingDropDieNum => {
                if character.is_digit(10) {
                    let drop_die = &new_roll.drop_die;
                    match drop_die {
                        Some(y) => {
                            match y {
                                DropDie::DropLowest(x) => {
                                    new_roll.drop_die = Some(DropDie::DropLowest(x * 10 + character.to_digit(10).unwrap() as usize));
                                }
                                DropDie::DropHighest(x) => {
                                    new_roll.drop_die = Some(DropDie::DropHighest(x * 10 + character.to_digit(10).unwrap() as usize));
                                }
                            }
                        }
                        None => {
                            return Err("Somehow got to ObtainingDropDieNum state without first getting to ObtainingDropDieType".into());
                        }
                    }
                }
                else {
                    rolls.push(new_roll);
                    new_roll = Roll::new();
                    state = States::ObtainingNextOperation;
                }
            }

            States::ObtainingNextOperation => {
                match character {
                    ' ' => {
                        continue;
                    }
                    '+' => {
                        state = States::ObtainingNumberOfDice;
                    }
                    '-' => {
                        new_roll.multiplier = -1;
                        state = States::ObtainingNumberOfDice;
                    }
                    _ => {
                        println!("{}", help_message);
                        return Err("Unknown character encountered".into());
                    }
                }
            }
        }
    }
    rolls.push(new_roll);
    Ok(rolls)
}

pub fn consume_input_to_output(input: &String, help_message: &String, skip_dropped: bool, short_output: bool, colour: bool) -> Result<String, Box<dyn std::error::Error>> {
    let mut rolls = consume_input_to_rolls(&input, &help_message)?;
    let mut rng = rand::rng();
    let mut running_total: i64 = 0;
    // And then we build the output
    let mut output: String = String::new();
    for (index, roll) in rolls.iter_mut().enumerate() {
        if short_output {
            add_to_total(roll, &mut running_total, &mut rng)?;
        }
        else {
            add_to_total_and_output(index, roll, &mut running_total, &mut output, &mut rng, skip_dropped, colour)?;
        }
    }
    // Specify what the result of our rolls were
    output += &(" => ".to_owned() + &running_total.to_string());
    if short_output {
        return Ok(running_total.to_string());
    }
    else {
        return Ok(output);
    }
}

#[cfg_attr(feature = "wasm", wasm_bindgen)]
// No error output in WASM
pub fn consume_input_to_output_without_error(input: String, help_message: String, skip_dropped: bool, short_output: bool) -> String {
    let output = consume_input_to_output(&input, &help_message, skip_dropped, short_output, false);
    match output {
        Ok(x) => {
            return x;
        }
        Err(y) => {
            return y.to_string();
        }
    }
}
