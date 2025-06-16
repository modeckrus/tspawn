use tspawn::{tspawn, A};

#[tokio::main]
async fn main() {
    let a = A::new(5);

    // This should expand to your desired pattern
    tspawn!(mut a, {
        *a += 1;
        println!("Updated a: {}", *a);
    })
    .await
    .ok();

    println!("Final value: {}", a.get());
}
