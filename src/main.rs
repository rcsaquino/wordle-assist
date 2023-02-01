use serde_json;
use std::io;
use std::io::Write;
use std::sync::mpsc;
use std::thread;
use std::thread::available_parallelism;

fn main() {
    use std::time::Instant;

    let mut possible_answers: Vec<String> =
        serde_json::from_str(include_str!("../db_words/wordle_full.json")).unwrap();

    println!("Wordle Assist v1.0.1");
    println!("------------------------------------------------------");
    println!("Type words in this format: [aaaaa-00000]");
    println!("0: Wrong, 1: Partial, 2: Exact");
    println!("------------------------------------------------------");

    let mut total_guess = 0;

    // Check best guess
    loop {
        let t0 = Instant::now();

        let mut priority_index = vec![0; possible_answers.len()];

        let thread_count = available_parallelism().unwrap().get();
        let chunk_size: usize =
            (possible_answers.len() as f64 / thread_count as f64).ceil() as usize;

        let rx = {
            let (tx, rx) = mpsc::channel();
            for chunk in 0..thread_count {
                let from = chunk_size * chunk;
                let to = {
                    if (from + chunk_size) > possible_answers.len() {
                        possible_answers.len()
                    } else {
                        from + chunk_size
                    }
                };
                let possible_answers = possible_answers.clone();
                let tx = tx.clone();
                thread::spawn(move || {
                    let mut priority_index = vec![0; possible_answers.len()];
                    for x in from..to {
                        for y in 0..possible_answers.len() {
                            for a_char_index in 0..5 {
                                let a_char = possible_answers[x].chars().nth(a_char_index).unwrap();
                                if a_char == possible_answers[y].chars().nth(a_char_index).unwrap()
                                {
                                    priority_index[y] += 5;
                                } else if possible_answers[y].contains(a_char) {
                                    priority_index[y] += 4;
                                }
                            }
                        }
                    }

                    tx.send(priority_index).unwrap();
                });
            }
            rx
        };

        for r in rx {
            for (i, prio) in r.iter().enumerate() {
                priority_index[i] += prio;
            }
        }

        let mut word_hash_vec = possible_answers
            .clone()
            .into_iter()
            .zip(priority_index.into_iter())
            .collect::<Vec<(String, u32)>>();

        word_hash_vec.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        println!(
            "Best words: {:?}",
            &word_hash_vec[if possible_answers.len() >= 5 {
                0..5
            } else {
                0..possible_answers.len()
            }]
        );

        println!("Process took {:.2} seconds.", t0.elapsed().as_secs_f64());
        println!("------------------------------------------------------");

        let mut user_input = String::new();
        print!("Enter [aaaaa-00000]: ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut user_input).unwrap();

        let user_input = user_input.trim();
        let user_input_split: Vec<&str> = user_input.split('-').collect();
        let word = user_input_split[0];
        let score = user_input_split[1];

        let mut present_chars = Vec::new();
        let mut absent_chars = Vec::new();

        for i in 0..5 {
            if score.chars().nth(i).unwrap() == '2' {
                present_chars.push(word.chars().nth(i).unwrap());

                possible_answers.retain(|answer| {
                    answer.chars().nth(i).unwrap() == word.chars().nth(i).unwrap()
                        && answer.contains(word.chars().nth(i).unwrap())
                });
            }
            if score.chars().nth(i).unwrap() == '1' {
                present_chars.push(word.chars().nth(i).unwrap());

                possible_answers.retain(|answer| {
                    answer.chars().nth(i).unwrap() != word.chars().nth(i).unwrap()
                        && answer.contains(word.chars().nth(i).unwrap())
                });
            }
            if score.chars().nth(i).unwrap() == '0' {
                absent_chars.push(word.chars().nth(i).unwrap());
            }
        }

        for char in absent_chars {
            if !present_chars.contains(&char) {
                possible_answers.retain(|answer| !answer.contains(char));
            }
        }

        possible_answers.retain(|answer| answer != &word);

        total_guess += 1;
        println!("Total guesses: {}/6", total_guess);
        if possible_answers.len() == 0 || total_guess == 6 {
            restart("Wordle solved!");
            break;
        } else if total_guess == 6 {
            restart("Wordle failed!");
            break;
        }
    }
}

fn restart(msg: &str) {
    println!("{}", msg);
    let mut user_input = String::new();
    print!("Restart? (y/n): ");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut user_input).unwrap();
    if user_input.trim() == String::from("y") {
        main();
    }
}
