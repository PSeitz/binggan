use binggan::{black_box, BenchGroup, GibKekseJetzt};

fn fibonacci(n: u64) -> u64 {
    match n {
        0 | 1 => 1,
        n => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

fn bench_fibonacci_group(runner: &mut BenchGroup) {
    // preparation code
    let val = 10;
    runner.register(
        "fibonacci",
        Box::new(move || {
            fibonacci(black_box(val));
        }),
    );
}

//fn bench_fibonacci_direct(runner: &mut GibKekseJetzt) {
//// preparation code
//let val = 10;
//runner.register(
//"fibonacci",
//Box::new(move || {
//fibonacci(black_box(val));
//}),
//);
//}

fn main() {
    let mut runner = GibKekseJetzt::new();
    bench_fibonacci_group(runner.new_group("fibonacci"));
    //bench_fibonacci_direct(&mut runner);
    runner.run();
    runner.report();
}
