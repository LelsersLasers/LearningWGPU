use advanced_wgpu::run;

fn main() {
    println!("Starting...");

    pollster::block_on(run());

    println!("Exiting...");
}
