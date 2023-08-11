use std::io;

const PI: f32 = 3.1415926;

#[derive(Debug)]
struct Rectangle {
    width: u32,
    height: u32,
}

impl Rectangle {
    // associated function
    fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    // asscociated function
    fn square(side: u32) -> Self {
        Self {
            width: side,
            height: side,
        }
    }

    // method
    fn width(&self) -> u32 {
        self.width
    }

    // method
    fn height(&self) -> u32 {
        self.height
    }

    // method
    fn area(&self) -> u32 {
        self.width() * self.height()
    }
}

#[derive(Debug)]
struct Triangle {
    base: u32,
    height: u32,
}

impl Triangle {
    fn new(base: u32, height: u32) -> Self {
        Self { base, height }
    }

    fn base(&self) -> u32 {
        self.base
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn area(&self) -> u32 {
        (self.base() * self.height()) / 2
    }
}

#[derive(Debug)]
struct Circle {
    radius: u32,
}

impl Circle {
    fn new(radius: u32) -> Self {
        Self { radius }
    }

    fn radius(&self) -> u32 {
        self.radius
    }

    fn area(&self) -> f32 {
        PI * self.radius().pow(2) as f32
    }
}

fn main() {
    loop {
        let mut sel = String::new();
        println!();
        println!("==============================================");
        println!("rec: rectangle");
        println!("sq: square");
        println!("tri: triangle");
        println!("cir: circle");
        println!("==============================================");
        println!();
        println!("Select one of the above shape to calculate the area (rec/sq/tri/cir):");
        get_std_input(&mut sel);
        println!("%%%%%%%%%%%%%%%%%%%%%%%%%%%%");
        match &sel.trim().to_lowercase()[..] {
            "rec" => {
                let rec = Rectangle::new(
                    ask_for_input("width", "rectangle"),
                    ask_for_input("height", "rectangle"),
                );
                print_result("rectangle", &rec.area().to_string()[..]);
            }
            "sq" => {
                let sq = Rectangle::square(ask_for_input("side", "square"));
                print_result("square", &sq.area().to_string()[..]);
            }
            "tri" => {
                let tri = Triangle::new(
                    ask_for_input("base", "triangle"),
                    ask_for_input("height", "triangle"),
                );
                print_result("triangle", &tri.area().to_string()[..]);
            }
            "cir" => {
                let cir = Circle::new(ask_for_input("radius", "circle"));
                print_result("circle", &cir.area().to_string()[..]);
            }
            _ => {
                println!("Invalid selection.");
                println!();
                continue;
            }
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

fn ask_for_input(prompt: &str, shape: &str) -> u32 {
    loop {
        let mut input = String::new();
        println!("What is the {} of the {}: ", prompt, shape);
        get_std_input(&mut input);
        let input: u32 = match input.trim().parse() {
            Ok(num) => num,
            Err(_) => {
                println!("Invalid {} for {}", prompt, shape);
                continue;
            }
        };
        return input;
    }
}

fn print_result(shape: &str, area: &str) {
    println!("The area of the {} is {}.", shape, area);
    println!();
}

fn get_std_input(str: &mut String) {
    io::stdin().read_line(str).expect("Failed to read line");
}
