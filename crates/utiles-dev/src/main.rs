// fn main() {
//     println!("utiles ~ dev");
// }

#[tokio::main]
async fn main() {
    println!("utiles ~ dev");

    let r = utiles_dev::quick_maths();
    if let Err(e) = r {
        println!("e: {:?}", e);
    } else {
        println!("2 + 2, that's 4, minus 1 that's 3, quick-maths.");
    }
}
