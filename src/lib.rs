// #![feature(trait_alias)]
use itertools::iproduct;

// TODO would be nice to cover the whole things with test, but looks like it end up with only smoke test (running and looks from afar like what was intended)
pub mod directions;

use crate::directions::{coordinate::Coordinate, direction::Direction};

use std::{
    collections::HashMap,
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct Planet {
    pub coordinate: Coordinate,
    pub weight: i32,
}

impl GravityObject for Planet {
    fn coordinate(&self) -> Coordinate {
        self.coordinate
    }
    fn coordinate_mut(&mut self) -> &mut Coordinate {
        &mut self.coordinate
    }
    fn as_gravity_receiver_mut(&mut self) -> Option<&mut dyn GravityReceiver> {
        None
    }
    fn as_gravity_source(&self) -> Option<&dyn GravitySource> {
        Some(self)
    }
}
impl GravitySource for Planet {
    fn weight(&self) -> i32 {
        self.weight
    }
    fn weight_mut(&mut self) -> i32 {
        self.weight
    }
}

#[derive(Clone)]
pub struct Asteroid {
    pub coordinate: Coordinate,
    pub velocity: Direction,
    delta: Direction,
}
impl Asteroid {
    pub fn new(coordinate: Coordinate, velocity: Direction) -> Asteroid {
        Asteroid {
            coordinate,
            velocity,
            delta: Default::default(),
        }
    }
}
impl GravityObject for Asteroid {
    fn coordinate(&self) -> Coordinate {
        self.coordinate
    }
    fn coordinate_mut(&mut self) -> &mut Coordinate {
        &mut self.coordinate
    }
    fn as_gravity_receiver_mut(&mut self) -> Option<&mut dyn GravityReceiver> {
        Some(self)
    }
    fn as_gravity_source(&self) -> Option<&dyn GravitySource> {
        None
    }
}
impl GravityReceiver for Asteroid {
    fn velocity(&self) -> &Direction {
        &self.velocity
    }
    fn velocity_mut(&mut self) -> &mut Direction {
        &mut self.velocity
    }
    fn delta_get(&self) -> Direction {
        self.delta
    }
    fn delta_update(&mut self, add_value: Direction) {
        self.delta += add_value;
    }
    fn delta_reset(&mut self) {
        self.delta = Default::default();
    }
}

pub trait GravityReceiver {
    fn velocity_mut(&mut self) -> &mut Direction;
    fn velocity(&self) -> &Direction;
    /// Accrued effect on velocity of the receiver.
    fn delta_get(&self) -> Direction;
    /// Add `add_value` to accrued effect on velocity.
    fn delta_update(&mut self, add_value: Direction);
    /// Reset velocity accrued effect.
    fn delta_reset(&mut self);
}
pub trait GravitySource {
    fn weight_mut(&mut self) -> i32;
    fn weight(&self) -> i32;
}
// pub trait GravityObject = GravityObjectBase + Clone;
pub trait GravityObject {
    fn coordinate_mut(&mut self) -> &mut Coordinate;
    fn coordinate(&self) -> Coordinate;
    fn as_gravity_source(&self) -> Option<&dyn GravitySource>;
    fn as_gravity_receiver_mut(&mut self) -> Option<&mut dyn GravityReceiver>;
}

fn get_distance(x1: i32, y1: i32, x2: i32, y2: i32) -> i32 {
    (((x1 - x2) * (x1 - x2) + (y1 - y2) * (y1 - y2)) as f64).sqrt() as i32
}

fn apply_physics(
    mut objects: Vec<Box<dyn GravityObject>>,
    gravitational_constant: i32,
) -> Vec<Box<dyn GravityObject>> {
    // Go through each pair of objects, and apply
    /*      The time has come to make it more honest implementation; which will be very naive and lead to combinatorial
    explosion pretty quickly. */
    /* I don't want to introduce `Clone` to `dyn` object as it requires to add a new compositional trait, so let's just
    workaround with an amendment list */
    let mut changeslist: HashMap<usize, Direction> = Default::default();
    /* Approach with nested for-loops failed due to requirement to `Clone` `objects` again. So let's apply a dirty hack with
    a pair of indecies. */
    // for receiving_obj in objects { // iproduct!(objects, objects) {
    let objects_len = objects.len();
    for (r, s) in iproduct!(0..objects_len, 0..objects_len) {
        let r_coordinate = objects[r].coordinate();
        let s_coordinate = objects[s].coordinate();
        // if let Some(receiving) = &mut objects[r].as_gravity_receiver_mut() {
        if objects[r].as_gravity_receiver_mut().is_some() {
            // for sourcing_obj in objects {
            // avoid self by coord since gravity doesn't affect if the coord equal anyway
            if r_coordinate != s_coordinate {
                // let mut deltaforce: Direction;
                if let Some(sourcing) = objects[s].as_gravity_source() {
                    let distance_sq = get_distance(
                        objects[s].coordinate().x,
                        objects[s].coordinate().y,
                        objects[r].coordinate().x,
                        objects[r].coordinate().y,
                    )
                    .pow(2);
                    let deltaforce = Direction {
                        x: (objects[r].coordinate().x - objects[s].coordinate().x)
                            * sourcing.weight()
                            * gravitational_constant
                            / distance_sq,
                        y: (objects[r].coordinate().y - objects[s].coordinate().y)
                            * sourcing.weight()
                            * gravitational_constant
                            / distance_sq,
                    };
                    let entry_the = changeslist.entry(r).or_default();
                    entry_the.x -= deltaforce.x;
                    entry_the.y -= deltaforce.y;
                }
                // receiving.delta_update(deltaforce);
            }
            // }
        }
    }
    for d in changeslist {
        objects[d.0]
            .as_gravity_receiver_mut()
            .expect("receivers have been filtered while producing the `HashMap`")
            .delta_update(d.1);
    }
    for object in &mut objects {
        let mut shift = Direction::default();
        if let Some(receiving) = object.as_gravity_receiver_mut() {
            let d = receiving.delta_get();
            let mut vel = receiving.velocity_mut();
            vel += d;
            receiving.delta_reset();
            shift = *receiving.velocity();
        }
        let object_coordinate = object.coordinate_mut();
        object_coordinate.x += shift.x;
        object_coordinate.y += shift.y;
    }

    objects
}

fn handle_connection(
    mut stream: TcpStream,
    mut objects: Vec<Box<dyn GravityObject>>,
    gravitational_constant: i32,
) -> Vec<Box<dyn GravityObject>> {
    objects = apply_physics(objects, gravitational_constant);

    #[derive(Deserialize, Serialize)]
    struct Circle {
        cx: i32,
        cy: i32,
        r: i32,
        stroke: String,
        fill: String,
        #[serde(rename = "stroke-width")]
        stroke_width: i32,
    }

    let get_circle = |o: &Box<dyn GravityObject>| -> Circle {
        Circle {
            cx: o.coordinate().x,
            cy: o.coordinate().y,
            r: match o.as_gravity_source() {
                Some(o) => o.weight(),
                None => 2,
            },
            stroke: "green".to_string(),
            fill: "black".to_string(),
            stroke_width: 3,
        }
    };
    let circles = objects.iter().map(|o| get_circle(o)).collect::<Vec<_>>();
    let contents = serde_json::to_string(&circles).unwrap();
    let status_line = "HTTP/1.1 200 OK";
    let response = format!(
        "{status_line}\r\nContentType: application/json\r\nAccess-Control-Allow-Origin: *\r\n\r\n{contents}\r\n"
    );
    stream.write_all(response.as_bytes()).unwrap();
    stream.flush().unwrap();
    stream.shutdown(std::net::Shutdown::Both).unwrap();

    objects //.into_iter().map(|o| o.as_object()).collect::<Vec<Box<dyn Object>>>()
}

pub fn start_server(
    uri: &str,
    mut objects: Vec<Box<dyn GravityObject>>,
    gravitational_constant: i32,
) -> ! {
    let listener = TcpListener::bind(uri).unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        objects = handle_connection(stream, objects, gravitational_constant);
    }

    unreachable!()
}
