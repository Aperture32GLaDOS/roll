# Roll
A rust program to roll dice, heavily inspired by [dice](https://github.com/ncfavier/dice)

Made primarily for Linux-based systems, everything should work on windows except maybe the ANSI color printing (use at your own peril!!!)

## Usage
- To roll a die, use ./roll d20 (replacing 20 with the number of sides)
- To roll many dice, use ./roll 2d20 (replacing 2 with the number of dice, and 20 with each die's number of sides)
- These expressions may be summed or subtracted (i.e. d20 + 3d4
- To roll and disregard the N lowest results, use ./roll 2d20dl1 (replacing 2 with the total number of rolls, and 1 for the number of dice to be discarded)
- Similarly, ./roll 2d20dh1 may be used
- In these cases, the discarded rolls will be displayed in red to mark them as dropped
