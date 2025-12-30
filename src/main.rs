use clap::{command, Arg};
use rand::Rng;

#[derive(Debug)]
struct Roll {
    // The number of dice to be rolled
    num_rolls: usize,
    // The sides of each die
    dice_sides: u8,
    // i.e. is this being added or subtracted from our running total?
    multiplier: i8,
    // Do we drop the highest, lowest, or no dice
    // (i.e. for advantage/disadvantage in DnD)
    drop_die: Option<DropDie>,
    // The results of each individual roll
    results: Vec<u32>
}

impl Roll {
    fn new() -> Self {
        Roll { num_rolls: 0, dice_sides: 0, multiplier: 1, drop_die: None, results: Vec::new() }
    }
}

// The states of our FSM, used to consume the user input
enum States {
    ObtainingNumberOfDice,
    ObtainingDiceSides,
    ObtainingDropDieType,
    ObtainingDropDieNum,
    ObtainingNextOperation
}

#[derive(Debug)]
enum DropDie {
    DropLowest(usize),
    DropHighest(usize)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut command = command!().arg(Arg::new("dice").num_args(1..).help("The dice to be rolled - syntax is NdN, and many dice may be summed or subtracted.\nIt is also possible to drop the N lowest or highest results, i.e. with 2d20dl1 or 2d20dh1. The dropped dice will be marked in \x1b[0;91mred\x1b[0m"));
    command.build();
    let help_message = command.render_long_help();
    let matches = command.get_matches();
    let mut rng = rand::rng();
    // Use a finite state machine approach to consume the input
    let input;
    if let Some(matches_found) = matches.get_many::<String>("dice") {
        input = matches_found.fold(String::new(), |v, x| v + " " + x);
    }
    else {
        println!("{}", help_message);
        return Ok(());
    }
    let mut state: States = States::ObtainingNumberOfDice;
    let mut rolls: Vec<Roll> = Vec::new();
    let mut new_roll = Roll::new();
    // While there is still input to consume...
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
                            return Ok(());
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
                            return Ok(());
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
                        return Ok(());
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
                        return Ok(());
                    }
                }
            }
        }
    }
    rolls.push(new_roll);
    // We have consumed the input, so we roll the dice...
    let mut running_total: i64 = 0;
    // And then we build the output
    let mut output: String = String::new();
    for (index, roll) in rolls.iter_mut().enumerate() {
        // Roll the dice
        if roll.num_rolls == 0 {
            continue;
        }
        roll.results.reserve(roll.num_rolls);
        for _ in 0..roll.num_rolls {
            roll.results.push(rng.random_range(1..=roll.dice_sides).into())
        }
        if index > 0 {
            match roll.multiplier {
                1 => {
                    output += " + ";
                }
                -1 => {
                    output += " - ";
                }
                _ => {
                    return Err("Multiplier somehow obtained as neither 1 or -1".into());
                }
            }
        }
        if roll.num_rolls == 1 {
            output += &roll.results.first().unwrap().to_string();
        }
        // If we are dropping, then...
        match &roll.drop_die {
            Some(y) => {
                match y {
                    DropDie::DropLowest(x) => {
                        if *x >= roll.num_rolls {
                            continue;
                        }
                        roll.results.sort_by_key(|n| *n);
                        if roll.num_rolls > 1 {
                            output.push('('); 
                            for (result_index, result) in roll.results.iter().enumerate() {
                                if result_index > 0 {
                                    output += " + "
                                }
                                if result_index < *x {
                                    output += "\x1b[0;91mX\x1b[0m";
                                }
                                output += &result.to_string();
                            }
                            output.push(')');
                        }
                        roll.results.drain(0..*x);
                    }
                    DropDie::DropHighest(x) => {
                        if *x >= roll.num_rolls {
                            continue;
                        }
                        roll.results.sort_by_key(|n| *n);
                        roll.results.reverse();
                        if roll.num_rolls > 1 {
                            output.push('('); 
                            for (result_index, result) in roll.results.iter().enumerate() {
                                if result_index > 0 {
                                    output += " + "
                                }
                                if result_index < *x {
                                    output += "\x1b[0;91mX\x1b[0m";
                                }
                                output += &result.to_string();
                            }
                            output.push(')');
                        }
                        roll.results.drain(0..*x);
                    }
                }
            }
            None => {
                if roll.num_rolls > 1 {
                    output.push('('); 
                    for (result_index, result) in roll.results.iter().enumerate() {
                        if result_index > 0 {
                            output += " + "
                        }
                        output += &result.to_string();
                    }
                    output.push(')');
                }
            }
        }
        running_total += ((roll.multiplier as i32) * (roll.results.iter().sum::<u32>() as i32)) as i64;
    }
    // Specify what the result of our rolls were
    output += &(" => ".to_owned() + &running_total.to_string());
    println!("{}", output);
    Ok(())
}
