use rand::prelude::*;
use std::io::{self, Write};
use std::process;
use std::cmp::Ordering;

/// The hand value at which the player is awarded a Blackjack.
const BLACKJACK: u32 = 21;

// TODO: Call these CARD_NAMES and change code to refer to this data as "Card Name"
/// All possible card values.
const CARD_VALUES: [&str; 13] = ["Ace", "Two", "Three", "Four", "Five", "Six", "Seven", "Eight", "Nine", "Ten", "Jack", "Queen", "King"];

/// All possible card suits.
const CARD_SUITS: [&str; 4] = ["Hearts", "Diamonds", "Clubs", "Spades"];

/// The hand value at which the dealer will stop trying to add to their own hand.
const DEALER_LIMIT: u32 = 17;

/// Used to print the help menu for each game phase.
const HELP_MENU_HEADER: [&str; 4] = [
    "-----------------------",
    "Command Line Blackjack",
    "-----------------------",
    "Available commands:",
];

/// The command line prompt used to indicate the user that the game is waiting for input.
const PROMPT: &str = ">>>";

/// The number of chips the player starts with.
const START_CHIPS: u32 = 10;

type Card = (String, String);
type Hand = Vec<Card>;
type Deck = Hand;

struct GameState {
    bet: u32,
    chips: u32,
    deck: Deck,
    hand: Hand,
    phase: GamePhase,
    rng: ThreadRng,
}

#[derive(PartialEq)]
enum GamePhase {
    OutOfGame,
    InGame,
}

/// Print the given lines to stdout.
fn print_lines(lines: Vec<&str>) {
    for line in lines.iter() {
        println!("{}", line);
    }
}

/// Print a help menu with the given available commands.
fn print_help_menu(commands: Vec<&str>) {
    print_lines(HELP_MENU_HEADER.to_vec());
    print_lines(commands);
}

/// Print a message that displays the cards in, and the value of the hand.
fn print_hand(hand: &Hand) {
    println!("(down) {}", get_card_name(&hand[0]));
    for card in hand[1..].iter() {
        println!("(up) {}", get_card_name(card));
    }
    println!("\nHand has a value of {}!", get_hand_value(hand));
}

/// Create and return a new shuffled Deck.
fn build_deck(rng: &mut ThreadRng) -> Deck {
    let mut deck = Vec::new();
    for value in CARD_VALUES.iter() {
        for suit in CARD_SUITS.iter() {
            deck.push((String::from(*value), String::from(*suit)));
        }
    }
    deck.shuffle(rng);
    return deck;
}

/// Return the integer value of the given hand.
fn get_hand_value(hand: &Hand) -> u32 {
    let mut total = 0;
    for (value, _) in hand {
        let mut value = (CARD_VALUES.iter().position(|v| v.eq(value)).unwrap() as u32) + 1;
        if value > 10 { value = 10; }
        else if value == 1 && total + 11 <= BLACKJACK { value = 11; }
        total += value;
    }
    total
}

/// Returns the name of the given card.
fn get_card_name(card: &Card) -> String {
    format!("{} of {}", card.0, card.1)
}

/// Tries to start a new hand, optionally with the given cards.
fn try_start_hand(state: &mut GameState, wager: u32, start_hand: Option<Hand>) {
    state.deck = build_deck(&mut state.rng);
    match start_hand {
        // TODO: Remove cards from the newly build deck?
        Some(hand) => { state.hand = hand; },
        None => {
            state.hand.clear();
            let down = state.deck.pop().unwrap();
            let up = state.deck.pop().unwrap();
            state.hand.push(down.clone());
            state.hand.push(up.clone());
        },
    };

    println!("You have been dealt the following cards:");
    print_hand(&state.hand);

    if get_hand_value(&state.hand) == BLACKJACK {
        println!("\nBLACKJACK!!!");
        state.chips += wager;
        return;
    }

    state.phase = GamePhase::InGame;
    state.bet = wager;
    state.chips -= wager;
    println!("\nTo view available commands, type \"help\".");
}

#[allow(dead_code)]
/// Parses a command line identifier of a card.
/// form: Ace_Hearts (note: currently case sensitive)
fn parse_card_id(id: &String) -> Result<Card, String> {
    let tokens = id.split('_').collect::<Vec<&str>>();
    if tokens.len() != 2 {
        return Err(format!("{} is not in the correct form Card_Suit!", id.clone()));
    }

    let name_index = CARD_VALUES.iter().position(|v| v.eq(&tokens[0]));
    let suit_index = CARD_SUITS.iter().position(|s| s.eq(&tokens[1]));

    if name_index.is_none() { return Err(format!("{} is not a valid value for card name!", tokens[0])); }
    if suit_index.is_none() { return Err(format!("{} is not a valid value for card suit!", tokens[1])); }

    Ok((String::from(tokens[0]), String::from(tokens[1])))
}

fn main() {
    let mut input_buffer = String::new();
    let mut state = GameState {
        bet: 0,
        chips: START_CHIPS,
        deck: Vec::new(),
        hand: Vec::new(),
        phase: GamePhase::OutOfGame,
        rng: rand::thread_rng(),
    };
    loop {
        if state.phase == GamePhase::OutOfGame && state.chips == 0 {
            print_lines(vec![
                "",
                "---------------------------------------------------",
                "You lost all of your chips! Come back later, chump!",
                "---------------------------------------------------",
            ]);
            process::exit(0);
        }

        input_buffer.clear();
        print!("{} ", PROMPT);
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut input_buffer).expect("Failed to read input!");
        let tokens = input_buffer.trim().split(' ').collect::<Vec<&str>>();

        match state.phase {
            GamePhase::OutOfGame => {
                #[cfg(feature="debug")]
                if tokens[0] == "debug-start" {
                    if tokens.len() != 3 {
                        println!("Usage: debug-start <card 1> <card 2>");
                        continue;
                    }
                    let card1 = parse_card_id(&String::from(tokens[1]));
                    let card2 = parse_card_id(&String::from(tokens[2]));

                    if card1.is_err() {
                        println!("Error: {}", card1.unwrap_err());
                        continue;
                    }
                    if card2.is_err() {
                        println!("Error: {}", card2.unwrap_err());
                        continue;
                    }

                    // These are safe unwraps because we have already checked the errors above.
                    try_start_hand(&mut state, 0, Some(vec![card1.unwrap(), card2.unwrap()]));
                    continue;
                }

                match tokens[0] {
                    "chips" => { println!("{}", state.chips); },
                    "exit" => { process::exit(0); },
                    "help" => {
                        print_help_menu(vec![
                            "chips: Show how many chips you have left.",
                            "exit: End the game.",
                            "start <wager>: Start a new hand with the given wager.",
                        ]);
                    },
                    "start" => {
                        if tokens.len() != 2 {
                            println!("Usage: start <wager>");
                            continue;
                        }
                        let wager = match tokens[1].parse::<u32>() {
                            Ok(wager) => wager,
                            Err(error) => {
                                println!("Error: unable to parse wager value - {}", error);
                                continue;
                            }
                        };
                        if wager == 0 {
                            println!("You must wager at least 1 chip!");
                            continue;
                        }
                        if wager > state.chips {
                            println!("You cannot wager {} chips because you only have {}!", wager, state.chips);
                            continue;
                        }

                        try_start_hand(&mut state, wager, None);
                    },
                    _ => println!("Invalid command!"),
                };
            },
            GamePhase::InGame => {
                match tokens[0] {
                    "exit" => { process::exit(0); },
                    "hand" => { print_hand(&state.hand); },
                    "help" => {
                        print_help_menu(vec![
                            "exit: End the game.",
                            "hand: View the cards you currently have in your hand.",
                            "hit: Have the dealer give you another card. Don't go over 21, though!",
                            "leave: Leave the current hand.",
                            "stay: Stop taking new cards and lock in your current hand value.",
                        ]);
                    },
                    "hit" => {
                        let card = state.deck.pop().unwrap();
                        state.hand.push(card.clone());
                        println!("You have been dealt the {}!\n", get_card_name(&card));
                        print_hand(&state.hand);

                        let hand_value = get_hand_value(&state.hand);
                        if hand_value > BLACKJACK {
                            println!("\nYOU BUSTED!!!");
                            state.phase = GamePhase::OutOfGame;
                        } else if hand_value == BLACKJACK {
                            println!("\nBLACKJACK!");
                            state.chips += state.bet;
                            state.phase = GamePhase::OutOfGame;
                        }
                    },
                    "leave" => {
                        state.phase = GamePhase::OutOfGame;
                        println!("You have quit your hand!");
                    },
                    "stay" => {
                        let mut dealer_hand = Vec::new();
                        dealer_hand.push(state.deck.pop().unwrap());
                        dealer_hand.push(state.deck.pop().unwrap());

                        // TODO: Will dealer try to beat the player's hand?
                        while get_hand_value(&dealer_hand) < DEALER_LIMIT {
                            dealer_hand.push(state.deck.pop().unwrap());
                        }

                        // TODO: Make helper function for surrounding with dashes.
                        print_lines(vec!["---------", "Your Hand", "---------"]);
                        print_hand(&state.hand);
                        print_lines(vec!["-------------", "Dealer's Hand", "-------------"]);
                        print_hand(&dealer_hand);
                        println!();

                        if get_hand_value(&dealer_hand) > BLACKJACK {
                            println!("DEALER BUSTED. YOU WIN!!!");
                            state.chips += state.bet * 2;
                        } else {
                            match get_hand_value(&dealer_hand).cmp(&get_hand_value(&state.hand)) {
                                Ordering::Less => {
                                    println!("YOU WIN!!!");
                                    state.chips += state.bet * 2;
                                },
                                Ordering::Equal => {
                                    println!("DRAW!!!");
                                    state.chips += state.bet;
                                },
                                Ordering::Greater => {
                                    println!("YOU LOSE!!!");
                                },
                            }
                        }

                        state.phase = GamePhase::OutOfGame;
                    },
                    _ => println!("Invalid command!"),
                }
            },
        }
    }
}