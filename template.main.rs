use std::env;

fn add(x: i32, y: i32) -> i32 {
    x + y
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    if args.len() < 3 {
        eprintln!("usage: {} <from> <to>", program);
        return;
    }

    print!(
        "{}",
        add(
            args[1].parse::<i32>().unwrap(),
            args[2].parse::<i32>().unwrap()
        )
    );
}
