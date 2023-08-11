
use openweather::LocationSpecifier;
use std::io;

fn main() {
    println!("I'll tell you the plan of our trip to Oregon, iff you tell me whether it's sunny on Friday, Mar 24.");
    println!();
    let mut input = String::new();

    loop {
        println!("Will it be SUNNY on Friday, Mar 24?  [yes / no] :  ");
        io::stdin().read_line(&mut input).expect("Fail to readline");

        input.make_ascii_lowercase();
        if input == "yes" || input.chars().nth(0).unwrap() == 'y' {
            spring_break_trip(true);
            break;
        } else if input == "no" || input.chars().nth(0).unwrap() == 'n' {
            spring_break_trip(false);
            break;
        } else {
            continue;
        }
    }
}

 fn spring_break_trip (is_friday_sunny: bool) {
  let stay: &'static str = "Cannon Beach / Seaside";
  let plan1: &'static str = "Visit Portland & new laptop for Seb";
  let plan2: &'static str = "Visit the beaches (Cannon Beach/Seaside)";
  let route_depart1: &'static str = "I-5 South (Seattle -> Portland)";
  let route_depart2: &'static str = "I-5 South + US-101 South (Seattle -> Seaside/Cannon Beach)";
  let route_por_to_beach: &'static str = "US-26 West (Portland -> Cannon Beach/Seaside)";
  let route_beach_to_por: &'static str = "US-26 East (Cannon Beach/Seaside -> Portland)";
  let route_return1: &'static str = "I-5 North (Portland -> Seattle)";
  let route_return2: &'static str = "US-101 North + I-5 North (Seaside/Cannon Beach -> Seattle)";

println!();
println!(">>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>");
  if !is_friday_sunny {
    println!("Friday Plan:");
    println!("  -- {}", route_depart1);
    println!("  -- {}", plan1);
    println!("  -- {}", route_por_to_beach);
    println!();

    println!("Staying at:");
    println!("  -- {}", stay);
    println!();

    println!("Saturday Plan:");
    println!("  -- {}", plan2);
    println!("  -- {}", route_return2);
  } else {
    println!("Friday Plan:");
    println!("  -- {}", route_depart2);
    println!("  -- {}", plan2);
    println!();

    println!("Staying at:");
    println!("  -- {}", stay);
    println!();

    println!("Saturday Plan:");
    println!("  -- {}", route_beach_to_por);
    println!("  -- {}", plan1);
    println!("  -- {}", route_return1);
  }
  println!(">>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>");
 }