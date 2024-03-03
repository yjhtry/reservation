use std::sync::Once;

static START: Once = Once::new();
fn main() {
    call_once();
    call_once();
}

fn call_once() {
    START.call_once(|| println!("222222"))
}
