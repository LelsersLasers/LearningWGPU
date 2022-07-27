use hello_triangle::run;

fn main() {
    println!("Starting...");

    pollster::block_on(run());

    println!("Exiting...");
}
