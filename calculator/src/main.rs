use std::env::{args, Args};

fn main() {
  let mut args: Args = args();

  let first = args.nth(1).unwrap();
  let operator = args.nth(2).unwrap();
  let second = args.nth(3).unwrap();
  println!("{:?} {} {}", first, operator, second);
}
