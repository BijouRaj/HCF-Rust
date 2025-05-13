// Cards are represented as integers from 0-51
// Diamonds: 0-12, Clubs: 13-25, Hearts: 26-38, Spades: 39-51
// 2-Ace in each suit: 0/13/26/39 = 2, 12/25/38/51 = Ace
// Deck class is for all deck related functions

use std::collections::HashMap;
use rand::seq::SliceRandom;
// thread_rng for compatibility
use rand::thread_rng;

// Use of constants instead of random numbers so operations are easier to correspond to the operations
const RANKS: usize = 13; // 13 ranks (card values)
const SUITS: usize = 4;  // 4 suits
const DECK_SIZE: usize = 52; // 52 Cards in deck
const HAND_SIZE: usize = 7; // 7 cards per hand
const NUM_PLAYERS: usize = 6; // 6 player team
const DEALER_CARDS: usize = 10; // 10 remaining unkown possible dealer cards

/// Converts a card integer (0-51) to a string representation
fn tostr(card: usize) -> String {
    let rank = card % RANKS;
    let suit = card / RANKS;
    let rank_char = match rank {
        0 => '2', 1 => '3', 2 => '4', 3 => '5', 4 => '6',
        5 => '7', 6 => '8', 7 => '9', 8 => 'T', // Ten
        9 => 'J', 10 => 'Q', 11 => 'K', 12 => 'A',
        _ => panic!("Invalid rank"),
    };
    let suit_char = match suit {
        0 => 'd',
        1 => 'c',
        2 => 'h',
        3 => 's',
        _ => panic!("Invalid suit"),
    };
    format!("{}{}", rank_char, suit_char)
}

/// Converts an array of card integers to an array of string representations
fn arr_to_strings<const N: usize>(cards: &[usize; N]) -> [String; N] {
    let mut result = std::array::from_fn(|_| String::new());
    for i in 0..N {
        result[i] = tostr(cards[i]);
    }
    result
}
fn vec_to_strings(cards: &[usize]) -> Vec<String> {
    cards.iter().map(|&card| tostr(card)).collect()
}

/// Deck struct for 52 card deck functions
struct Deck {
    cards: [usize; DECK_SIZE],
}
impl Deck {
    fn new(existing_deck: Option<[usize; DECK_SIZE]>) -> Self {
        match existing_deck {
            Some(cards) => Deck { cards },
            None => {
                let mut cards = [0; DECK_SIZE];
                for i in 0..DECK_SIZE {
                    cards[i] = i;
                }
                let mut deck = Deck { cards };
                deck.shuffle();
                deck
            }
        }
    }
    fn shuffle(&mut self) {
        let mut rng = thread_rng();
        self.cards.shuffle(&mut rng);
    }
    fn get_cards(&self) -> [usize; DECK_SIZE] {
        self.cards
    }
    fn get_player_hands(&self) -> [[usize; HAND_SIZE]; NUM_PLAYERS] {
        let mut hands = [[0; HAND_SIZE]; NUM_PLAYERS];
        for player in 0..NUM_PLAYERS {
            for card in 0..HAND_SIZE {
                let deck_index = player * HAND_SIZE + card;
                hands[player][card] = self.cards[deck_index];
            }
        }
        hands
    }
    fn get_dealer_cards(&self) -> [usize; DEALER_CARDS] {
        let mut dealer_cards = [0; DEALER_CARDS];
        for i in 0..DEALER_CARDS {
            dealer_cards[i] = self.cards[NUM_PLAYERS * HAND_SIZE + i];
        }
        dealer_cards
    }
}

/// Returns the flush cards dynamic array sorted by rank (high to low)
fn get_best_flush<const N: usize>(hand: &[usize; N]) -> Vec<usize> {
    let mut suits: HashMap<usize, Vec<usize>> = HashMap::new();
    for &card in hand {
        let suit = card / RANKS;
        suits.entry(suit).or_insert(Vec::new()).push(card);
    }
    for (_, cards) in suits.iter_mut() {
        cards.sort_by(|&a, &b| {
            let rank_a = a % RANKS;
            let rank_b = b % RANKS;
            rank_b.cmp(&rank_a)
        });
    }
    let best_flush = suits.values()
        .max_by(|a, b| {
            let len_cmp = a.len().cmp(&b.len());
            if len_cmp != std::cmp::Ordering::Equal {
                return len_cmp;
            }
            for i in 0..a.len().min(b.len()) {
                let rank_a = a[i] % RANKS;
                let rank_b = b[i] % RANKS;
                let rank_cmp = rank_a.cmp(&rank_b);
                if rank_cmp != std::cmp::Ordering::Equal {
                    return rank_cmp;
                }
            }
            std::cmp::Ordering::Equal
        })
        .unwrap_or(&Vec::new())
        .clone();
    best_flush
}

/// Compares player and dealer hands, returns net gain/loss in antes
/// Player hand is ALWAYS the first parameter, dealer hand is the second
fn compare_hands<const P: usize, const D: usize>(
    player_hand: &[usize; P], 
    dealer_hand: &[usize; D]
) -> i32 {
    let player_flush = get_best_flush(player_hand);
    let dealer_flush = get_best_flush(dealer_hand);
    let play_bet_multiplier = match player_flush.len() {
        0..=4 => 1,
        5 => 2,
        _ => 3,
    };
    let dealer_qualifies = dealer_flush.len() >= 4 || (dealer_flush.len() == 3 && (dealer_flush[0] % RANKS) >= 7);
    if !dealer_qualifies {
        return 1;
    }
    if player_flush.len() > dealer_flush.len() {
        return 1 + play_bet_multiplier;
    } else if player_flush.len() < dealer_flush.len() {
        return -(1 + play_bet_multiplier);
    }
    for i in 0..player_flush.len().min(dealer_flush.len()) {
        let player_rank = player_flush[i] % RANKS;
        let dealer_rank = dealer_flush[i] % RANKS;
        
        if player_rank > dealer_rank {
            return 1 + play_bet_multiplier;
        } else if player_rank < dealer_rank {
            return -(1 + play_bet_multiplier);
        }
    }
    return 0
}

/// Calculates the average wager result across all possible dealer hands given the 10 remaining dealer cards and the player's 7 cards
/// Player hand is ALWAYS the first parameter, dealer cards is the second
fn calculate_average_result(
    player_cards: &[usize; HAND_SIZE], 
    dealer_cards: &[usize; DEALER_CARDS]
) -> f64 {
    let mut total_result = 0;
    let mut count = 0;
    let mut current = [0; HAND_SIZE];
    generate_and_process_combinations(
        dealer_cards, 
        0, 
        &mut current, 
        0, 
        player_cards,
        &mut total_result,
        &mut count
    );
    total_result as f64 / count as f64
}

/// Helper function for calculate_average_result()
fn generate_and_process_combinations<const N: usize>(
    arr: &[usize; N],
    start: usize,
    current: &mut [usize; HAND_SIZE],
    depth: usize,
    player_cards: &[usize; HAND_SIZE],
    total_result: &mut i32,
    count: &mut i32
) {
    if depth == HAND_SIZE {
        let result = compare_hands(player_cards, current);
        *total_result += result;
        *count += 1;
        return;
    }
    for i in start..N {
        current[depth] = arr[i];
        generate_and_process_combinations(
            arr, i + 1, current, depth + 1, 
            player_cards, total_result, count
        );
    }
}

fn test_functionality() {
    // Card to string
    assert_eq!(tostr(0), "2d");
    assert_eq!(tostr(12), "Ad");
    assert_eq!(tostr(24), "Kc");  // King of Clubs
    assert_eq!(tostr(51), "As");
    let cards = [0, 13, 26, 39];
    assert_eq!(arr_to_strings(&cards), ["2d", "2c", "2h", "2s"]);
    
    // flush identification
    let hand = [12, 11, 9, 25, 24, 23, 40]; // 3-card diamond flush and 3-card clubs flush, clubs higher
    let flush = get_best_flush(&hand);
    assert_eq!(flush.len(), 3);
    assert_eq!(flush[0] / RANKS, 1); // Clubs
    
    // Test hand comparison
    let player = [39, 40, 41, 42, 51, 5, 18]; // 5-card spade flush
    let dealer = [26, 27, 28, 29, 4, 17, 30]; // 5-card heart flush
    let result = compare_hands(&player, &dealer);
    assert_eq!(result, 3);
    
    // Test hand comparison with non-qualified dealer
    let dealer_low = [0, 1, 2, 15, 16, 30, 40]; // 3-card diamond flush, too low
    let result2 = compare_hands(&player, &dealer_low);
    assert_eq!(result2, 1);

    // Test dealer average with less than 3-card flush
    let dealer_no_flush = [0, 1, 2, 13, 14, 15, 26, 27, 28, 39]; // Not qualified
    let result3 = calculate_average_result(&player, &dealer_no_flush);
    assert_eq!(result3, 1.0);
    
    // Test average result calc
    let test_player = [0, 1, 2, 3, 4, 5, 6]; // 7 card flush
    let test_dealer = [12,1,2,25,14,15,16,26,27,39];
    let avg = calculate_average_result(&test_player, &test_dealer);
    println!("Sample average result: {}; Tests successful", avg);
}

// End of program base structure and classes


// perfect collusion

pub fn perfect_collusion_sim(num_simulations: usize) {
    let mut total_score = 0.0;

    for _ in 0..num_simulations {
        let deck = Deck::new(None);
        let hands = deck.get_player_hands();
        let dealer_cards = deck.get_dealer_cards();

        for player_hand in hands.iter() {
            let avg_result = calculate_average_result(player_hand, &dealer_cards);
            if avg_result > -1.0 {
                total_score += avg_result;
            } else {
                total_score += -1.0;
            }
        }
    }

    let total_hands = (num_simulations * NUM_PLAYERS) as f64;
    let avg_per_hand = total_score / total_hands;

    println!(
        "Perfect Collusion Strategy Results:\n\
        Total Simulated Hands: {}\n\
        Total Winnings: {:.2}\n\
        Average Winnings per Hand: {:.4}",
        total_hands,
        total_score,
        avg_per_hand
    );
}

// end of perfect collusion

// no collusion losing optimal strategy (mousseau)

pub fn simulate_mousseau_strategy(iterations: usize) {
    let mut deck = Deck::new(None);
    let mut total_winnings: f64 = 0.0;

    for _ in 0..iterations {
        deck.shuffle();
        let players_hands = deck.get_player_hands();
        let dealer_hand = deck.get_dealer_cards();

        for player in players_hands.iter() {
            let raise_multiplier = mousseau_strategy(player);

            if raise_multiplier == 0 {
                total_winnings -= 1.0; // Player folds, loses ante
            } else {
                total_winnings += calculate_average_result(player, &dealer_hand);
            }
        }
    }

    let total_wagers = (iterations * 6) as f64;
    let average_per_wager = total_winnings / total_wagers;
    
    println!(
        "Mousseau Strategy Results:\n\
        Total Simulated Hands: {}\n\
        Total Winnings: {:.2}\n\
        Average Winnings per Wager: {:.4}",
        total_wagers,
        total_winnings,
        average_per_wager
    );
}

// Determines raise multiplier (0 = fold, 1–3 = raise) based on the Mousseau non-collusion strategy
fn mousseau_strategy<const N: usize>(hand: &[usize; N]) -> u8 {
    let flush = get_best_flush(hand);
    let flush_len = flush.len();

    match flush_len {
        4 => 1,
        5 => 2,
        6 | 7 => 3,
        3 => {
            let mut ranks: Vec<usize> = flush.iter().map(|&card| card % RANKS).collect();
            ranks.sort_unstable_by(|a, b| b.cmp(a)); // Descending

            if ranks[0] >= 8 && ranks[1] >= 6 && ranks[2] >= 4 {
                1
            } else {
                0
            }
        }
        _ => 0,
    }
}
// end of mosseau

// beginning of e jacobson

pub fn ap_heat(iterations : usize) -> f64
{
    let mut deck = Deck::new(None);
    let mut total_winnings : f64 = 0.0;

    for i in 0..iterations
    {   // Reset Hands
        deck.shuffle();
        let players_hands = deck.get_player_hands();
        let dealer_hand = deck.get_dealer_cards();

        // Finds the play/fold strategy for the round depending on remaining suits
        let mut suit_counts = [0; SUITS];
        for hand in players_hands.iter()
        {
            for card in hand.iter()
            {
                suit_counts[card / RANKS] += 1;
            }
        }
        suit_counts = suit_counts.map(|x| RANKS - x);
        suit_counts.sort();
        let strategy = get_strategy(suit_counts);

        // Each player bets or folds based on their hand and round strategy
        for player in players_hands.iter()
        {
            if should_play(get_best_flush(player), strategy)
            {
                total_winnings += calculate_average_result(player, &dealer_hand);
            }
            else
            {
                total_winnings -= 1.0;
            }
        }
    }

    // The expected average winning per hand for an individual player
    total_winnings / (iterations * NUM_PLAYERS) as f64
}

// Returns the strategy, represented by a number based on the number of suits
// remaining in the dealer's potential hand
fn get_strategy(signals : [usize; SUITS]) -> usize
{   // Derived from table used in https://www.888casino.com/blog/novelty-games/high-card-flush-collusion
    match signals[0]
    {
        0 => match signals[1]
        {
            0 => match signals[2]
            {
                0|1 => 7,
                _ => 5
            },
            1 => match signals[2]
            {
                1 => 6,
                2 => 5,
                _ => 4
            },
            2 => match signals[2]
            {
                2 => 4,
                _ => 11
            },
            _ => 10
        },
        1 => match signals[1]
        {
            1 => match signals[2]
            {
                1 => 5,
                2 => 4,
                _ => 11
            },
            2 => match signals[2]
            {
                2 => 10,
                _ => 9
            },
            _ => 8
        },
        2 => 12,
        _ => panic!()
    }
}

// Compares the flush given to see if the player should play it
// based on the strategy given
fn should_play(flush : Vec<usize>, strategy : usize) -> bool
{
    match strategy
    {
        4..=7 => flush.len() >= strategy,
        8..=11 => flush.len() > 3 || (flush.len() == 3 && flush[0] % RANKS >= strategy),
        12 => true,
        _ => panic!()
    }
}

// end of jacobson

fn main() {
    test_functionality();
    perfect_collusion_sim(1000000);
    simulate_mousseau_strategy(1000000);
    println!("Eliot Jacobson average net profit per wager: {}", ap_heat(1000000))
}