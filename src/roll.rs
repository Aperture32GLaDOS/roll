use rand::{distr::Uniform, rngs::ThreadRng, RngExt};
use crate::tree::ASTInput;

#[derive(Debug, Clone, PartialEq)]
pub enum DropDie {
    DropLowest(usize),
    DropHighest(usize)
}

#[derive(Clone, PartialEq, Debug)]
pub enum RollOrConstant {
    Roll(Roll),
    Const(ConstantRoll)
}

#[derive(Debug, Clone, PartialEq)]
pub struct Roll {
    // The number of dice to be rolled
    pub num_rolls: usize,
    // The sides of each die
    pub dice_sides: u8,
    // Do we drop the highest, lowest, or no dice
    // (i.e. for advantage/disadvantage in DnD)
    pub drop_die: Option<DropDie>,
    // The results of each individual roll
    pub results: Vec<u32>
}

#[derive(Clone, PartialEq, Debug)]
pub struct ConstantRoll {
    pub constant_result: i64
}

impl Roll {
    pub fn new() -> Self {
        Roll { num_rolls: 0, dice_sides: 0, drop_die: None, results: Vec::new() }
    }
}

impl ConstantRoll {
    pub fn new() -> Self {
        ConstantRoll { constant_result: 0 }
    }
}

pub trait Rollable {
    // Add to a running total and build up an output string
    fn roll_with_output(&mut self, output: &mut String, rng: &mut ThreadRng, skip_dropped: bool, colour: bool) -> Result<i64, Box<dyn std::error::Error>>;
    fn roll(&mut self, rng: &mut ThreadRng) -> Result<i64, Box<dyn std::error::Error>>;
}

impl Rollable for ConstantRoll {
    fn roll_with_output(&mut self, output: &mut String, _rng: &mut ThreadRng, _skip_dropped: bool, _colour: bool) -> Result<i64, Box<dyn std::error::Error>> {
        *output += &self.constant_result.to_string();
        Ok(self.constant_result)
    }

    fn roll(&mut self, _rng: &mut ThreadRng) -> Result<i64, Box<dyn std::error::Error>> {
        Ok(self.constant_result)
    }
}

impl Rollable for Roll {
    fn roll_with_output(&mut self, output: &mut String, rng: &mut ThreadRng, skip_dropped: bool, colour: bool) -> Result<i64, Box<dyn std::error::Error>> {
        // Roll the dice
        if self.num_rolls == 0 {
            return Ok(0);
        }
        self.results.reserve(self.num_rolls);
        let distribution = Uniform::new_inclusive(1, self.dice_sides)?;
        for _ in 0..self.num_rolls {
            self.results.push(rng.sample(distribution).into())
        }
        if self.num_rolls == 1 {
            *output += &self.results.first().unwrap().to_string();
        }
        // If we are dropping, then...
        match &self.drop_die {
            Some(y) => {
                let drop_n: usize;
                match y {
                    DropDie::DropLowest(x) => {
                        if *x >= self.num_rolls {
                            return Ok(0);
                        }
                        self.results.sort_by_key(|n| *n);
                        drop_n = *x;
                        
                    }
                    DropDie::DropHighest(x) => {
                        if *x >= self.num_rolls {
                            return Ok(0);
                        }
                        self.results.sort_by_key(|n| *n);
                        self.results.reverse();
                        drop_n = *x;
                    }
                }
                // If skipping,
                if skip_dropped {
                    // Start by removing the dropped rolls
                    self.results.drain(0..drop_n);
                    // And then add to output string
                    if self.results.len() > 1 {
                        output.push('('); 
                        for (result_index, result) in self.results.iter().enumerate() {
                            if result_index > 0 {
                                *output += " + "
                            }
                            *output += &result.to_string();
                        }
                        output.push(')');
                    }
                    else {
                        for (result_index, result) in self.results.iter().enumerate() {
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
                    if self.num_rolls > 1 && !skip_dropped {
                        output.push('('); 
                        for (result_index, result) in self.results.iter().enumerate() {
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
                    self.results.drain(0..drop_n);
                }
            }
            None => {
                if self.num_rolls > 1 {
                    output.push('('); 
                    for (result_index, result) in self.results.iter().enumerate() {
                        if result_index > 0 {
                            *output += " + "
                        }
                        *output += &result.to_string();
                    }
                    output.push(')');
                }
            }
        }
        Ok((self.results.iter().sum::<u32>() as i32) as i64)
    }

    fn roll(&mut self, rng: &mut ThreadRng) -> Result<i64, Box<dyn std::error::Error>> {
        // Roll the dice
        if self.num_rolls == 0 {
            return Ok(0);
        }
        self.results.reserve(self.num_rolls);
        let distribution = Uniform::new_inclusive(1, self.dice_sides)?;
        for _ in 0..self.num_rolls {
            self.results.push(rng.sample(distribution).into())
        }
        // If we are dropping, then...
        match &self.drop_die {
            Some(y) => {
                let drop_n: usize;
                match y {
                    DropDie::DropLowest(x) => {
                        if *x >= self.num_rolls {
                            return Ok(0);
                        }
                        self.results.sort_by_key(|n| *n);
                        drop_n = *x;
                        
                    }
                    DropDie::DropHighest(x) => {
                        if *x >= self.num_rolls {
                            return Ok(0);
                        }
                        self.results.sort_by_key(|n| *n);
                        self.results.reverse();
                        drop_n = *x;
                    }
                }
                // And end with removing the dropped dice
                self.results.drain(0..drop_n);
            }
            None => {}
        }
        Ok((self.results.iter().sum::<u32>() as i32) as i64)
    }
}

impl Rollable for RollOrConstant {
    fn roll(&mut self, rng: &mut ThreadRng) -> Result<i64, Box<dyn std::error::Error>> {
        match self {
            Self::Roll(x) => x.roll(rng),
            Self::Const(x) => x.roll(rng)
        }
    }

    fn roll_with_output(&mut self, output: &mut String, rng: &mut ThreadRng, skip_dropped: bool, colour: bool) -> Result<i64, Box<dyn std::error::Error>> {
        match self {
            Self::Roll(x) => x.roll_with_output(output, rng, skip_dropped, colour),
            Self::Const(x) => x.roll_with_output(output, rng, skip_dropped, colour)
        }
    }
}

// The states of our FSM, used to consume the user input
pub enum States {
    ObtainingNumberOfDice,
    ObtainingDiceSides,
    ObtainingDropDieType,
    ObtainingDropDieNum,
}

pub fn consume_input_to_roll(input: &Vec<ASTInput>, index: &mut usize, help_message: &String) -> Result<RollOrConstant, Box<dyn std::error::Error>> {
    let mut new_roll = Roll::new();
    let mut new_const = ConstantRoll::new();
    let mut state: States = States::ObtainingNumberOfDice;
    // While loop so we can go back and forth between characters in our FSM
    while *index < input.len() {
        let ASTInput::Character(character) = input[*index] else {break};
        if character.is_whitespace() {continue}
        *index += 1;
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
                            return Ok(RollOrConstant::Roll(new_roll));
                        }
                        '+' => {
                            return Ok(RollOrConstant::Roll(new_roll));
                        }
                        '-' => {
                            return Ok(RollOrConstant::Roll(new_roll));
                        }
                        '*' => {
                            return Ok(RollOrConstant::Roll(new_roll));
                        }
                        '/' => {
                            return Ok(RollOrConstant::Roll(new_roll));
                        }
                        '^' => {
                            return Ok(RollOrConstant::Roll(new_roll));
                        }
                        'd' => {
                            state = States::ObtainingDropDieType;
                        }
                        _ => {
                            println!("{}", character);
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
                        // If we find a character we are not expecting, assume that this is a
                        // constant and parse the current character as part of the next operation
                        _ => {
                            new_const.constant_result = new_roll.num_rolls as i64;
                            return Ok(RollOrConstant::Const(new_const));
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
                    return Ok(RollOrConstant::Roll(new_roll));
                }
            }
        }
    }
    if new_roll.dice_sides == 0 {
        Ok(RollOrConstant::Const(ConstantRoll { constant_result: new_roll.num_rolls as i64 }))
    }
    else {
        Ok(RollOrConstant::Roll(new_roll))
    }
}
