use utiles_dev::cover::burn_main;
use utiles_dev::edges::edges_main;

fn main() {
    println!("utiles ~ dev");

    edges_main();

    burn_main();
}

// #[tokio::main]
// async fn main_async() {
//     println!("utiles ~ dev");

//     edges_main();

//     let r = utiles_dev::quick_maths();
//     if let Err(e) = r {
//         println!("e: {:?}", e);
//     } else {
//         println!("2 + 2, that's 4, minus 1 that's 3, quick-maths.");
//     }
// }
