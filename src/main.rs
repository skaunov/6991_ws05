use simulator_lib::directions::{coordinate::Coordinate, direction::Direction};
use simulator_lib::{start_server, Asteroid, GravityObject, Planet};
fn main() {
    let mut objects: Vec<Box<dyn GravityObject>> = vec![
        Box::new(Planet {
            coordinate: Coordinate::new(500, 500),
            weight: 50,
        }),
        Box::new(Asteroid::new(
            Coordinate::new(250, 250),
            Direction { x: 30, y: -40 },
        )),
        Box::new(Asteroid::new(
            Coordinate::new(750, 750),
            Direction { x: -30, y: 40 },
        )),
    ];

    println!("Starting server. Open phys_simulation.html to see the simulation.");
    start_server("localhost:16991", objects, 70);
}
