mod sim;
mod vis;
use vis::Visualization;
mod util;
fn main() {
    let euler = Visualization::new(200, 200);
    euler.run();
}
