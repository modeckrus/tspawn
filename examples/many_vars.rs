use tspawn::{tspawn, A};

#[tokio::main]
async fn main() {
    let a = A::new(0);
    let b = A::new(0);
    let c = A::new(0);
    let d = A::new(0);
    let e = A::new(0);
    let f = A::new(0);
    let g = A::new(0);
    tspawn!(a, mut b, ref c, d, ref e, mut f, g, {
        *b += 1;
        *f += 1;
        println!(
            "a: {}, b: {}, c: {}, d: {}, e: {}, f: {}, g: {}",
            *a.read(),
            *b,
            *c,
            *d.read(),
            *e,
            *f,
            *g.read()
        );
    })
    .await
    .ok();
}
