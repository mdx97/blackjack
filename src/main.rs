use rand::prelude::*;
use std::io::{self, Write};
use std::process;

const CARD_VALUES: [&str; 13] = ["Ace", "Two", "Three", "Four", "Five", "Six", "Seven", "Eight", "Nine", "Ten", "Jack", "Queen", "King"];
const CARD_SUITS: [&str; 4] = ["Hearts", "Diamonds", "Clubs", "Spades"];
const DEALER_LIMIT: u32 = 17;
const HELP_MENU_HEADER: [&str; 4] = [
    "-----------------------",
    "Command Line Blackjack",
    "-----------------------",
    "Available commands:",
];
const PROMPT: &str = ">>>";

type Card = (String, String);
type Hand = Vec<Card>;
type Deck = Hand;

#[derive(PartialEq)]
enum GamePhase {
    OutOfGame,
    InGame,
}

fn print_lines(lines: Vec<&str>) {
    for line in lines.iter() {
        println!("{}", line);
    }
}

fn print_help_menu(commands: Vec<&str>) {
    print_lines(HELP_MENU_HEADER.to_vec());
    print_lines(commands);
}

fn print_hand(hand: &Hand) {
    println!("(down) {}", get_card_name(&hand[0]));
    for card in hand[1..].iter() {
        println!("(up) {}", get_card_name(card));
    }
    println!("\nYour hand has a value of {}!", get_hand_value(hand));
}

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

fn get_hand_value(hand: &Hand) -> u32 {
    let mut total = 0;
    for (value, _) in hand {
        let mut value = (CARD_VALUES.iter().position(|v| v.eq(value)).unwrap() as u32) + 1;
        if value > 10 { value = 10; }
        // TODO: Handle the case where if we are busting, Aces can be worth 1 instead.
        if value == 1 { value = 11; }
        total += value;

    }
    total
}

fn get_card_name(card: &Card) -> String {
    format!("{} of {}", card.0, card.1)
}

fn main() {
    let mut rng = rand::thread_rng();
    let mut deck = Vec::new();
    let mut chips = 10;
    let mut input_buffer = String::new();
    let mut phase = GamePhase::OutOfGame;
    let mut hand = Vec::new();
    let mut bet = 0;

    loop {
        if phase == GamePhase::OutOfGame && chips == 0 {
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

        match phase {
            GamePhase::OutOfGame => {
                match tokens[0] {
                    "chips" => { println!("{}", chips); },
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
                        if wager > chips {
                            println!("You cannot wager {} chips because you only have {}!", wager, chips);
                            continue;
                        }

                        bet = wager;
                        chips -= wager;
                        phase = GamePhase::InGame;
                        deck = build_deck(&mut rng);
                        hand.clear();

                        // TODO: Handle blackjack off the draw.
                        let down = deck.pop().unwrap();
                        let up = deck.pop().unwrap();
                        hand.push(down.clone());
                        hand.push(up.clone());

                        println!("You have been dealt the following cards:");
                        print_hand(&hand);
                        println!("\nTo view available commands, type \"help\".");
                    },
                    _ => println!("Invalid command!"),
                };
            },
            GamePhase::InGame => {
                match tokens[0] {
                    "exit" => { process::exit(0); },
                    "hand" => { print_hand(&hand); },
                    "help" => {
                        print_help_menu(vec![
                            "exit: End the game.",
                            "hand: View the cards you currently have in your hand.",
                            "hit: Have the dealer give you another card. Don't go over 21, though!",
                            "leave: Leave the current hand."
                        ]);
                    },
                    "hit" => {
                        let card = deck.pop().unwrap();
                        hand.push(card.clone());
                        println!("You have been dealt the {}!\n", get_card_name(&card));
                        print_hand(&hand);

                        let hand_value = get_hand_value(&hand);
                        if hand_value > 21 {
                            println!("\nYOU BUSTED!!!");
                            phase = GamePhase::OutOfGame;
                        } else if hand_value == 21 {
                            println!("\nBLACKJACK!");
                            chips += bet;
                            phase = GamePhase::OutOfGame;
                        }
                    },
                    "leave" => {
                        phase = GamePhase::OutOfGame;
                        println!("You have quit your hand!");
                    },
                    _ => println!("Invalid command!"),
                }
            },
        }
    }
}