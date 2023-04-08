use std::cell::RefCell;
use std::rc::Rc;

use simulator_lib::directions::{coordinate::Coordinate, direction::Direction};
use simulator_lib::{start_server, Asteroid, Object, Planet};
#[derive(Clone)]
struct Distantoid {
    // I want to make this test with minimal changes to the `lib`, so it's a work around, that could be generalized into it. TODO think if it's possible at all to do that "trick" for more than one gravity source.
    /*      I come to the conclusion that it's impossible to have a reference to the gravity source,
    since `lib` is built around `&mut` and prevents even reading the object or its method.
            So it seems the only workaround I see now is to move the logic to the `Planet`, which
            isn't really satisfying, but I hope it will do the trick. */
    source_gr: Rc<RefCell<dyn Object>>, 
    coordinate: Coordinate, velocity: Direction
}
impl Distantoid {
    // As soon this `fn` wasn't made `pub` in `lib`, let's just copy-paste it from there. Looks like a good candidate for `directions`, btw.
    fn get_distance(x1: i32, y1: i32, x2: i32, y2: i32) -> i32 {
        (((x1 - x2) * (x1 - x2) + (y1 - y2) * (y1 - y2)) as f64).sqrt() as i32
    }
}
impl Object for Distantoid {
    fn is_gravity_source(&self) -> bool {false}
    fn is_gravity_receiver(&self) -> bool {
        Distantoid::get_distance(
            self.coordinate.x, self.coordinate.y, 
            self.source_gr.borrow().get_coordinate().x, self.source_gr.borrow().get_coordinate().y
        ) > 100
    }

    fn coordinate(&mut self) -> &mut Coordinate {&mut self.coordinate}
    fn get_coordinate(&self) -> Coordinate {self.coordinate}

    fn weight(&self) -> i32 {
        // TODO any graceful refactor?
        /*      check the code -- maybe it's feasible to return
                zero, but then it should be well-documented */
        panic!("should never be called on gravity non-source")
    }

    fn velocity(&mut self) -> &mut Direction {&mut self.velocity}
    fn get_velocity(&self) -> &Direction {&self.velocity}
}

fn main() {
    let source_gr = Rc::new(RefCell::new(Planet {
        coordinate: Coordinate::new(500, 500),
        weight: 50,
    }));
    let mut objects: Vec<Rc<RefCell<dyn Object>>> = vec![
        source_gr.clone(),
        Rc::new(RefCell::new(Distantoid {
            coordinate:Coordinate::new(250,250), velocity:Direction{x:30,y: -40}, 
            source_gr: source_gr.clone()
        })),
        Rc::new(RefCell::new(Distantoid {
            coordinate: Coordinate::new(750, 750),
            velocity: Direction { x: -30, y: 40 },
            source_gr: source_gr.clone()
        }))
    ];

    println!("Starting server. Open phys_simulation.html to see the simulation.");
    start_server("localhost:16991", objects, 70);
}
