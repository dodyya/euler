mod flip;
mod sim;
use flip::Flip;
mod util;
fn main() {
    let flip = Flip::new(125, 125);
    flip.run();
}
