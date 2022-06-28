extern "C" {
    fn Nas();
}

fn main() {
    unsafe { Nas() };
    println!("Hello, world!");
}
