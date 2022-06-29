#[repr(transparent)]
struct WaitGroup(*const i32);

extern "C" {
    fn Nas(_: WaitGroup);
    fn NewWaitGroup() -> WaitGroup;
}

fn main() {
    unsafe {
        let wg = NewWaitGroup();
        Nas(wg);
    };
    println!("Hello, world!");
}
