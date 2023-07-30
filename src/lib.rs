pub mod directions;

use crate::directions::{coordinate::Coordinate, direction::Direction};

use std::{
    cell::RefCell,
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    rc::Rc,
    thread,
    time::Duration,
};

use serde::{Deserialize, Serialize};

const BREAKING_VALUE_KIND: &str = "breaking usage of value kind for this type of `Object`";

#[derive(Clone)]
pub struct Planet {
    pub coordinate: Coordinate,
    pub weight: i32,
}

impl Planet {
    fn get_location(&self) -> Coordinate {
        self.coordinate.clone()
    }

    fn get_weight(&self) -> i32 {
        self.weight
    }
}
impl Object for Planet {
    fn is_gravity_source(&self) -> bool {
        true
    }
    fn is_gravity_receiver(&self) -> bool {
        false
    }
    fn coordinate(&mut self) -> &mut Coordinate {
        &mut self.coordinate
    }
    fn weight(&self) -> Option<i32> {
        Some(self.weight)
    }
    fn velocity(&mut self) -> Option<&mut Direction> {
        None
    }
    fn get_velocity(&self) -> Option<&Direction> {
        None
    }
    fn get_coordinate(&self) -> Coordinate {
        self.coordinate
    }
}

#[derive(Clone)]
pub struct Asteroid {
    pub coordinate: Coordinate,
    pub velocity: Direction,
}

impl Asteroid {
    fn get_location(&self) -> Coordinate {
        self.coordinate.clone()
    }

    fn get_velocity(&self) -> Direction {
        self.velocity.clone()
    }
}
impl Object for Asteroid {
    fn is_gravity_source(&self) -> bool {
        false
    }
    fn is_gravity_receiver(&self) -> bool {
        true
    }
    fn coordinate(&mut self) -> &mut Coordinate {
        &mut self.coordinate
    }
    fn weight(&self) -> Option<i32> {
        None
    }
    fn velocity(&mut self) -> Option<&mut Direction> {
        Some(&mut self.velocity)
    }
    fn get_coordinate(&self) -> Coordinate {
        self.coordinate
    }

    fn get_velocity(&self) -> Option<&Direction> {
        todo!()
    }
}

pub trait Object {
    fn is_gravity_source(&self) -> bool;
    fn is_gravity_receiver(&self) -> bool;
    fn coordinate(&mut self) -> &mut Coordinate;
    fn get_coordinate(&self) -> Coordinate;
    fn weight(&self) -> Option<i32>;
    fn velocity(&mut self) -> Option<&mut Direction>;
    fn get_velocity(&self) -> Option<&Direction>;
}

fn get_distance(x1: i32, y1: i32, x2: i32, y2: i32) -> i32 {
    (((x1 - x2) * (x1 - x2) + (y1 - y2) * (y1 - y2)) as f64).sqrt() as i32
}

fn apply_physics(
    mut objects: Vec<Rc<RefCell<dyn Object>>>,
    gravitational_constant: i32,
) -> Vec<Rc<RefCell<dyn Object>>> {
    // Go through each pair of objects, and apply
    let gravity_sources = objects
        .iter()
        .filter_map(|o| {
            return if o.borrow().is_gravity_source() {
                Some((
                    o.borrow().get_coordinate().clone(),
                    o.borrow().weight().expect(BREAKING_VALUE_KIND),
                ))
            } else {
                None
            };
        })
        .collect::<Vec<_>>();

    objects.iter_mut().for_each(|o/* : &mut Rc<RefCell<dyn Object>> */| {
        if o.borrow().is_gravity_receiver() {
            // ~~TODO understand why this assignment "breaking everyhting." I.e. mess `borrow()`.~~
            let asteroid = o;
            gravity_sources
                .iter()
                .for_each(|(planet_coord, planet_weight)| {
                    let distance = get_distance(
                        planet_coord.x,
                        planet_coord.y,
                        /* ~~TODO understand if it's possible to go here without mutability (which isn't used anyway)~~ */
                        //      preliminary answer is yes, but it would require to refactor whole function to be functional and ditch in-place update
                        asteroid.borrow().get_coordinate().x,
                        asteroid.borrow().get_coordinate().y,
                    );
                    let distance = distance * distance;

                    let force = Direction {
                        x: (asteroid.borrow().get_coordinate().x - planet_coord.x)
                            * planet_weight
                            * gravitational_constant
                            / distance,
                        y: (asteroid.borrow().get_coordinate().y - planet_coord.y)
                            * planet_weight
                            * gravitational_constant
                            / distance,
                    };
                    asteroid.borrow_mut().velocity().expect(BREAKING_VALUE_KIND).x -= force.x;
                    asteroid.borrow_mut().velocity().expect(BREAKING_VALUE_KIND).y -= force.y;

                    let vel = asteroid.borrow().get_velocity().expect(BREAKING_VALUE_KIND).clone();
                })
        }
    });

    // Apply the new velocity to each object.
    objects.iter_mut().for_each(|object| {
        if object.borrow().is_gravity_receiver() {
            let bx = object.borrow().get_velocity().expect(BREAKING_VALUE_KIND).x;
            let by = object.borrow().get_velocity().expect(BREAKING_VALUE_KIND).y;
            let mut m = object.borrow_mut()/* .coordinate() */;
            m.coordinate().x += bx;
            m.coordinate().y += by;
        }
    });

    objects
    // objects.into_iter().map(|o| o.as_object()).collect::<Vec<Box<dyn Object>>>()
}

fn handle_connection(
    mut stream: TcpStream,
    mut objects: Vec<Rc<RefCell<dyn Object>>>,
    gravitational_constant: i32,
) -> Vec<Rc<RefCell<dyn Object>>> {
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

    let get_circle = |o: &Rc<RefCell<dyn Object>>| -> Circle {
        match (
            o.borrow().is_gravity_source(),
            o.borrow().is_gravity_receiver(),
        ) {
            (true, false) => Circle {
                cx: o.borrow().get_coordinate().x,
                cy: o.borrow().get_coordinate().y,
                r: o.borrow().weight().expect(BREAKING_VALUE_KIND),
                stroke: "green".to_string(),
                fill: "black".to_string(),
                stroke_width: 3,
            },
            (false, true) => Circle {
                cx: o.borrow().get_coordinate().x,
                cy: o.borrow().get_coordinate().y,
                r: 2,
                stroke: "green".to_string(),
                fill: "black".to_string(),
                stroke_width: 3,
            },
            (true, true) => todo!(),
            (false, false) => todo!(),
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
    mut objects: Vec<Rc<RefCell<dyn Object>>>,
    gravitational_constant: i32,
) -> ! {
    let listener = TcpListener::bind(uri).unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        objects = handle_connection(stream, objects, gravitational_constant);
    }

    unreachable!()
}
