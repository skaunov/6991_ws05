use simulator_lib::directions::{coordinate::Coordinate, direction::Direction};
use simulator_lib::{start_server, Asteroid, Object, Planet};

#[derive(Clone)]
enum Pulse {
    Off,
    On,
}
#[derive(Clone)]
struct WeightPulsing {
    value: i32,
    counter: u8,
    state: Pulse,
}
#[derive(Clone)]
// I just liked using the name for the exercise, of course no resemblence to real <https://en.wikipedia.org/wiki/Pulsar>s.
struct Pulsar {
    coordinate: Coordinate,
    pub weight: WeightPulsing,
}
impl Object for Pulsar {
    fn is_gravity_source(&self) -> bool {
        true
    }
    fn is_gravity_receiver(&self) -> bool {
        false
    }

    fn coordinate(&mut self) -> &mut Coordinate {
        &mut self.coordinate
    }
    fn get_coordinate(&self) -> Coordinate {
        self.coordinate
    }

    fn weight(&mut self) -> Option<i32> {
        match self.weight.state {
            Pulse::Off => {
                if self.weight.counter >= 2 {
                    self.weight.state = Pulse::On
                };
                Some(0) // TODO having a coef here would be better, but let's keep the first step as simple as possible
            }
            Pulse::On => {
                if self.weight.counter >= 3 {
                    self.weight.state = Pulse::Off
                };
                Some(self.weight.value)
            }
        }
    }
    fn get_weight(&self) -> Option<i32> {
        match self.weight.state {
            Pulse::Off => Some(0),
            Pulse::On => Some(self.weight.value),
        }
    }

    fn velocity(&mut self) -> Option<&mut Direction> {
        None
    }
}

fn main() {
    let mut objects: Vec<Box<dyn Object>> = vec![
        Box::new(Pulsar {
            coordinate: Coordinate::new(500, 500),
            weight: WeightPulsing {
                value: 100,
                counter: Default::default(),
                state: Pulse::On,
            },
        }),
        Box::new(Asteroid {
            coordinate: Coordinate::new(250, 250),
            velocity: Direction { x: 30, y: -40 },
        }),
        Box::new(Asteroid {
            coordinate: Coordinate::new(750, 750),
            velocity: Direction { x: -30, y: 40 },
        }),
    ];

    println!("Starting server. Open phys_simulation.html to see the simulation.");
    start_server("localhost:16991", objects, 70);
}
