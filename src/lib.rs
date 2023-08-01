// TODO would be nice to cover the whole things with test, but looks like it end up with only smoke test (running and looks from afar like what was intended)
pub mod directions;

use crate::directions::{coordinate::Coordinate, direction::Direction};

use std::{
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

impl GravitySource for Planet {
    // fn is_gravity_source(&self) -> bool {true}
    // fn is_gravity_receiver(&self) -> bool {false}
    fn coordinate(&mut self) -> &mut Coordinate {
        &mut self.coordinate
    }
    fn get_coordinate(&self) -> Coordinate {
        self.coordinate
    }
    fn weight(&mut self) -> i32 {
        self.weight
    }
    fn get_weight(&self) -> i32 {
        self.weight
    }
}
impl GravityObject for Planet {
    fn one(&self) -> Option<&dyn GravitySource> {
        Some(self)
    }
    fn other(&self) -> Option<&dyn GravityReceiver> {
        None
    }
    fn receiver_mut(&mut self) -> Option<&mut dyn GravityReceiver> {
        None
    }
}

#[derive(Clone)]
pub struct Asteroid {
    pub coordinate: Coordinate,
    pub velocity: Direction,
}

impl GravityReceiver for Asteroid {
    // fn is_gravity_source(&self) -> bool {false}
    // fn is_gravity_receiver(&self) -> bool {true}
    fn coordinate(&mut self) -> &mut Coordinate {
        &mut self.coordinate
    }
    fn get_coordinate(&self) -> Coordinate {
        self.coordinate
    }
    fn velocity(&mut self) -> &mut Direction {
        &mut self.velocity
    }
    fn get_velocity(&self) -> &Direction {
        &self.velocity
    }
}
impl GravityObject for Asteroid {
    fn one(&self) -> Option<&dyn GravitySource> {
        None
    }
    fn other(&self) -> Option<&dyn GravityReceiver> {
        Some(self)
    }
    fn receiver_mut(&mut self) -> Option<&mut dyn GravityReceiver> {
        Some(self)
    }
}

// ~~TODO check privacy preservation against initial code~~
pub trait GravityReceiver {
    fn coordinate(&mut self) -> &mut Coordinate;
    fn get_coordinate(&self) -> Coordinate;
    fn velocity(&mut self) -> &mut Direction;
    fn get_velocity(&self) -> &Direction;
}
pub trait GravitySource {
    fn coordinate(&mut self) -> &mut Coordinate;
    fn get_coordinate(&self) -> Coordinate;
    fn weight(&mut self) -> i32;
    fn get_weight(&self) -> i32;
}
pub trait GravityObject {
    fn one(&self) -> Option<&dyn GravitySource>;
    fn other(&self) -> Option<&dyn GravityReceiver>;
    fn receiver_mut(&mut self) -> Option<&mut dyn GravityReceiver>;
}

fn get_distance(x1: i32, y1: i32, x2: i32, y2: i32) -> i32 {
    (((x1 - x2) * (x1 - x2) + (y1 - y2) * (y1 - y2)) as f64).sqrt() as i32
}

fn apply_physics(
    mut objects: Vec<Box<dyn GravityObject>>,
    gravitational_constant: i32,
) -> Vec<Box<dyn GravityObject>> {
    // Go through each pair of objects, and apply
    /*      ~~TODO~~ I still go with the assumption that `GravityObject` can be either `GravityReceiver` or `GravitySource`, though obvious next step would be break this assumption, and couple
    of `todo!()`s left in the trait are precisely for it. Though it would need a separate upgrade/refactoring. */
    /*          actually (thanks to `rebase`) the only couple of `todo()` left are in `handle_connection`;
    and even if being filled here's no type which have both kinds of `GravityObject` `trait` to experience such a luxury */
    let gravity_sources = objects
        .iter()
        .filter_map(|o| o.one().map(|o| (o.get_coordinate(), o.get_weight())))
        .collect::<Vec<_>>();

    objects.iter_mut().for_each(|o| {
        if let Some(o) = o.receiver_mut() {
            let asteroid = o;
            gravity_sources
                .iter()
                .for_each(|(planet_coord, planet_weight)| {
                    let planet_coord = planet_coord;
                    let distance = get_distance(
                        planet_coord.x,
                        planet_coord.y,
                        asteroid.coordinate().x,
                        asteroid.coordinate().y,
                    );
                    let distance = distance * distance;

                    let planet_weight = planet_weight;
                    let force = Direction {
                        x: (asteroid.coordinate().x - planet_coord.x)
                            * planet_weight
                            * gravitational_constant
                            / distance,
                        y: (asteroid.coordinate().y - planet_coord.y)
                            * planet_weight
                            * gravitational_constant
                            / distance,
                    };
                    asteroid.velocity().x -= force.x;
                    asteroid.velocity().y -= force.y;

                    let vel = asteroid.velocity();
                })
        }
    });

    // Apply the new velocity to each object.
    objects.iter_mut().for_each(|object| {
        if let Some(object) = object.receiver_mut() {
            object.coordinate().x += object.velocity().x;
            object.coordinate().y += object.velocity().y;
        }
    });

    objects
    // objects.into_iter().map(|o| o.as_object()).collect::<Vec<Box<dyn Object>>>()
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
        match (o.one(), o.other()) {
            (Some(o), None) => Circle {
                cx: o.get_coordinate().x,
                cy: o.get_coordinate().y,
                r: o.get_weight(),
                stroke: "green".to_string(),
                fill: "black".to_string(),
                stroke_width: 3,
            },
            (None, Some(o)) => Circle {
                cx: o.get_coordinate().x,
                cy: o.get_coordinate().y,
                r: 2,
                stroke: "green".to_string(),
                fill: "black".to_string(),
                stroke_width: 3,
            },
            (Some(_), Some(_)) => todo!(),
            (None, None) => todo!(),
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
