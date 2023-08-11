use median_pig_database::median;
fn main() {
    let list = median::IntList::new_from_stdin();
    println!("The median is: {}", list.get_median());
    println!("The mode is: {}", list.get_mode());
}
