use std::io;

enum Tempreture {
    C(i128),
    F(i128),
}

fn main() {
    loop {
        let mut sel = String::new();
        print_menu();
        println!("Select one of the above function to run (temp/sing/fib):");
        get_std_input(&mut sel);
        println!("%%%%%%%%%%%%%%%%%%%%%%%%%%%%");
        if sel.trim().to_lowercase() == "temp" {
            temp();
        } else if sel.trim().to_lowercase() == "sing" {
            the_twelve_days_of_christmas();
        } else if sel.trim().to_lowercase() == "fib" {
            fib();
        } else {
            println!("Invalid selection.");
            println!();
            continue;
        }
        println!("%%%%%%%%%%%%%%%%%%%%%%%%%%%%");
        loop {
            let mut quit = String::new();
            println!("Quit this program? (y/n):");
            get_std_input(&mut quit);
            if quit.trim().to_lowercase() == "y" {
                return;
            } else if quit.trim().to_lowercase() == "n" {
                break;
            } else {
                continue;
            }
        }
        println!();
    }
}

fn temp() {
    loop {
        let mut unit = String::new();
        let mut num = String::new();
        println!();
        println!("++++++++++++++++++++++++");
        println!("f: Fahrenheit");
        println!("c: Celsius");
        println!("++++++++++++++++++++++++");
        println!();
        println!("Select one of the unit listed above (f/c):");
        get_std_input(&mut unit);
        if !(unit.trim().to_lowercase() == "f") && !(unit.trim().to_lowercase() == "c") {
            println!("invalid selection");
            println!();
            continue;
        } else {
            println!("Please input the numeric value of the tempreture:");
            get_std_input(&mut num);
            let num: i128 = match num.trim().parse() {
                Ok(num) => num,
                Err(_) => {
                    println!("Invalid input.");
                    println!();
                    continue;
                }
            };
            if unit.trim().to_lowercase() == "f" {
                println!();
                println!("{num}℉ = {}℃", temp_convert(Tempreture::F(num)).to_string());
            } else {
                println!();
                println!("{num}℃ = {}℉", temp_convert(Tempreture::C(num)).to_string());
            }
        }
        println!();
        loop {
            let mut ans = String::new();
            println!("Go back to menu? (y/n):");
            get_std_input(&mut ans);
            if ans.trim().to_lowercase() == "y" {
                return;
            } else if ans.trim().to_lowercase() == "n" {
                break;
            } else {
                continue;
            }
        }
        println!();
    }
}

fn fib() {
    loop {
        let mut n = String::new();
        println!("Please enter a number n (0 <= n <= 186) for nth fibonacci number: ");
        get_std_input(&mut n);
        let n: u128 = match n.trim().parse() {
            Ok(num) => {
                if num > 186 {
                    println!("invalid number.");
                    println!();
                    continue;
                } else {
                    num
                }
            }
            Err(_) => {
                println!("invalid input.");
                println!();
                continue;
            }
        };
        println!(
            "The {n}{} fibonacci number is {}",
            match n {
                1 => "st",
                2 => "nd",
                3 => "rd",
                _ => "th",
            },
            fibonacci(n)
        );
        println!();
        loop {
            let mut ans = String::new();
            println!("Go back to menu? (y/n):");
            get_std_input(&mut ans);
            if ans.trim().to_lowercase() == "y" {
                return;
            } else if ans.trim().to_lowercase() == "n" {
                break;
            } else {
                continue;
            }
        }
        println!();
    }
}

fn print_menu() {
    println!();
    println!("==============================================");
    println!("temp: convert temperatures between Fahrenheit and Celsius.");
    println!("sing: Print the lyrics to the Christmas carol “The Twelve Days of Christmas.");
    println!("fib: generate the nth Fibonacci number.");
    println!("==============================================");
    println!();
}

fn the_twelve_days_of_christmas() {
    let lines = [
        "A partridge in a pear tree",
        "Two turtle doves, and",
        "Three french hens",
        "Four calling birds",
        "Five golden rings",
        "Six geese a-laying",
        "Seven swans a-swimming",
        "Eight maids a-milking",
        "Nine ladies dancing",
        "Ten lords a-leaping",
        "Eleven pipers piping",
        "Twelve drummers drumming",
    ];

    let days = [
        "first", "second", "third", "fourth", "fifth", "sixth", "seventh", "eighth", "ninth",
        "tenth", "eleventh", "twelfth",
    ];

    let mut current_day = 1;

    for day in days {
        println!("On the {} day of Christmas, my true love sent to me", day);
        for n in (1..=current_day).rev() {
            println!("{}", lines[n - 1]);
        }
        println!();
        current_day += 1;
    }
}

fn fibonacci(n: u128) -> u128 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci_helper(n - 1, 1, 0),
    }
}

fn fibonacci_helper(n: u128, cur: u128, prev: u128) -> u128 {
    if n == 0 {
        cur
    } else {
        fibonacci_helper(n - 1, cur + prev, cur)
    }
}

fn temp_convert(t: Tempreture) -> i128 {
    match t {
        Tempreture::C(num) => ((num * 9) / 5) + 32,
        Tempreture::F(num) => ((num - 32) * 5) / 9,
    }
}

fn get_std_input(str: &mut String) {
    io::stdin().read_line(str).expect("Failed to read line");
}
