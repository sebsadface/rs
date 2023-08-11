use rand::Rng;
use std::cmp::Ordering;
use std::io;

fn main() {
    println!("Guess the number!");

    loop {
        println!("Please input the guessing range:");

        println!("Minimum number (inclusive): ");
        let mut min = String::new();
        get_std_input(&mut min);
        let min: u32 = match min.trim().parse() {
            Ok(num) => num,
            Err(_) => {
                println!("Please enter a number");
                continue;
            }
        };

        println!("Maximum number (inclusive): ");
        let mut max = String::new();
        get_std_input(&mut max);
        let max: u32 = match max.trim().parse() {
            Ok(num) => num,
            Err(_) => {
                println!("Please enter a number");
                continue;
            }
        };

        let secret_number = rand::thread_rng().gen_range(min..=max);

        loop {
            println!("Please input your guess.");
            let mut guess = String::new();
            get_std_input(&mut guess);
            let guess: u32 = match guess.trim().parse() {
                Ok(num) => num,
                Err(_) => {
                    println!("Please enter a number!");
                    continue;
                }
            };

            println!("You guessed: {guess}");

            match guess.cmp(&secret_number) {
                Ordering::Less => println!("Too small!"),
                Ordering::Greater => println!("Too big!"),
                Ordering::Equal => {
                    println!("You win!");
                    break;
                }
            }
        }

        println!("Another round? (y/n):");
        let mut res = String::new();
        get_std_input(&mut res);
        match res.trim() {
            "n" => break,
            _ => continue,
        }
    }
}

fn get_std_input(str: &mut String) {
    io::stdin().read_line(str).expect("Failed to read line");
}
