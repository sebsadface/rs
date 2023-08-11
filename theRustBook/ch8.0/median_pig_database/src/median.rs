use std::collections::HashMap;
use std::io;

pub struct IntList {
    v: Vec<i32>, //sorted
    m: HashMap<i32, u32>,
}

impl IntList {
    pub fn new_from_stdin() -> Self {
        let mut m: HashMap<i32, u32> = HashMap::new();
        let mut v: Vec<i32> = Vec::new();
        let mut nums = String::new();
        println!("---------------------------------------------------------");
        println!("Please enter a list of integer separated by white spaces:");
        io::stdin()
            .read_line(&mut nums)
            .expect("Failed to read line");

        for token in nums.split_whitespace() {
            let num: i32 = match token.trim().parse() {
                Ok(num) => num,
                Err(_) => {
                    println!("Error parsing number: {}", token);
                    continue;
                }
            };
            v.push(num);
            let count = m.entry(num).or_insert(0);
            *count += 1;
        }

        println!();
        println!("Initializing IntList finished!");
        println!("---------------------------------------------------------");

        Self { v, m }
    }

    pub fn get_median(&self) -> i32 {
        let l = self.v.len();
        match l % 2 {
            0 => (self.v[(l / 2) - 1] + self.v[l / 2]) / 2,
            _ => self.v[l / 2],
        }
    }

    pub fn get_mode(&self) -> i32 {
        let mut max = (0, 0u32);

        for (n, count) in &self.m {
            if *count > max.1 {
                max = (*n, *count);
            }
        }

        max.0
    }
}
