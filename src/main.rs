use clap::{command, Arg, ArgAction};
use roll::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut command = command!()
        .arg(Arg::new("dice").num_args(1..).help("The dice to be rolled - syntax is NdN, and many dice may be summed or subtracted.\nIt is also possible to drop the N lowest or highest results, i.e. with 2d20dl1 or 2d20dh1. The dropped dice will be marked in \x1b[0;91mred\x1b[0m"))
        .arg(Arg::new("skip-dropped").short('s').long("skip-dropped").action(ArgAction::SetTrue).help("Do not show dice which have been dropped i.e. in 2d20dl1"))
        .arg(Arg::new("short-output").long("short-output").action(ArgAction::SetTrue).help("Only show the result of the rolls"))
        .long_about("Rolls dice for use in D&D");
    command.build();
    let help_message = command.render_long_help().to_string();
    let matches = command.get_matches();
    // Use a finite state machine approach to consume the input
    let input;
    if let Some(matches_found) = matches.get_many::<String>("dice") {
        input = matches_found.fold(String::new(), |v, x| v + " " + x);
    }
    else {
        println!("{}", help_message);
        return Ok(());
    }
    let skip_dropped = matches.get_flag("skip-dropped");
    let short_output = matches.get_flag("short-output");
    println!("{}", consume_input_to_output(input, help_message, skip_dropped, short_output, true)?);
    Ok(())
}
