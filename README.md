https://codereview.stackexchange.com/questions/286316/how-much-traits-are-enough-an-exercise-on-polymorphism

This is solution for [workshop]. I feel that I've take a slightly different path from the one intended by the authors (other WS was much more clear/straightforward to me except [one other]), so I'm asking to review my solution(s) to this one. Not to bloat the page too much I divide the solution according to the tasks into logical parts; this page contains the first part.

# solution and exercise text

Here you find the text of the solution, and copy of relevant exercise text from the [workshop] (to archive it and for convinience). It's also available as a [repo](https://github.com/skaunov/6991_ws05).

Couple of questions which are puzzling me are bolded out in the next section. Comments on any other topics you spot are very welcome!


## Solution
See https://github.com/skaunov/6991_ws05/compare/ffb3cdee5b2f00bc1d96f2115346f817c857e991..3ba793eecde103de229a3ef6a08f3afd5f5be5f6 for web compare tool; same diff is presented below. I guess just posting the code would demand you to involve too much in what was given as the starting point for the exercise; though this solution could be just run at `master` branch ref.

(Insignificant changes are ommitted.)
```diff
diff --git a/src/lib.rs b/src/lib.rs
index 31002b8..0b91715 100644
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -12,16 +12,7 @@ use std::{
 
 use serde::{Deserialize, Serialize};
 
-#[derive(Deserialize, Serialize)]
-struct Circle {
-    cx: i32,
-    cy: i32,
-    r: i32,
-    stroke: String,
-    fill: String,
-    #[serde(rename = "stroke-width")]
-    stroke_width: i32,
-}
+const BREAKING_VALUE_KIND: &str = "breaking usage of value kind for this type of `Object`";
 
 #[derive(Clone)]
 pub struct Planet {
```
This changes are unrelated to each other. `Circle` is moved into the `fn` which is the only one actually using it. In fact **it's one of the decisions I really would like to get feedback to**. Since on the one side it seems to be a reasonable move, on the other for a `lib` crate it could be not the best approach. On the third this one never been `pub` so maybe it's reasonable approach. 
Addition of the `const` is just to use it with `expect`s in the cases which shouldn't be reachable in the solution.

```diff
@@ -37,17 +28,14 @@ impl Planet {
     fn get_weight(&self) -> i32 {
         self.weight
     }
-
-    fn as_circle(&self) -> Circle {
-        Circle {
-            cx: self.coordinate.x,
-            cy: self.coordinate.y,
-            r: self.weight,
-            stroke: "green".to_string(),
-            fill: "black".to_string(),
-            stroke_width: 3,
-        }
-    }
+}
+impl Object for Planet {
+    fn is_gravity_source(&self) -> bool {true}
+    fn is_gravity_receiver(&self) -> bool {false}    
+    fn coordinate(&mut self) -> &mut Coordinate {&mut self.coordinate}
+    fn weight(&self) -> Option<i32> {Some(self.weight)}
+    fn velocity(&mut self) -> Option<&mut Direction> {None}
+    fn get_coordinate(&self) -> Coordinate {self.coordinate}
 }
 
 #[derive(Clone)]
@@ -64,45 +52,36 @@ impl Asteroid {
     fn get_velocity(&self) -> Direction {
         self.velocity.clone()
     }
-
-    fn as_circle(&self) -> Circle {
-        Circle {
-            cx: self.coordinate.x,
-            cy: self.coordinate.y,
-            r: 2,
-            stroke: "green".to_string(),
-            fill: "black".to_string(),
-            stroke_width: 3,
-        }
-    }
 }
-
-#[derive(Clone)]
-pub enum ObjectType {
-    Planet(Planet),
-    Asteroid(Asteroid),
```
According to the exercise `enum` is removed here; and `pub trait` introduced as the replacement.
```diff
+impl Object for Asteroid {
+    fn is_gravity_source(&self) -> bool {false}
+    fn is_gravity_receiver(&self) -> bool {true}
+    fn coordinate(&mut self) -> &mut Coordinate {&mut self.coordinate}
+    fn weight(&self) -> Option<i32> {None}
+    fn velocity(&mut self) -> Option<&mut Direction> {Some(&mut self.velocity)}
+    fn get_coordinate(&self) -> Coordinate {self.coordinate}
 }
 
-impl ObjectType {
-    fn get_circle(&self) -> Circle {
-        match self {
-            ObjectType::Planet(p) => p.as_circle(),
-            ObjectType::Asteroid(a) => a.as_circle(),
-        }
-    }
+pub trait Object {
+    fn is_gravity_source(&self) -> bool;
+    fn is_gravity_receiver(&self) -> bool;
+    fn coordinate(&mut self) -> &mut Coordinate;
+    fn get_coordinate(&self) -> Coordinate;
+    fn weight(&self) -> Option<i32>;
+    fn velocity(&mut self) -> Option<&mut Direction>;
 }
```

**Here's another decision I'd really like to get feedback on.** On the one hand I feel that capturing possible behavior setting `Option`s is inferior to separate the behavior into few traits (with supertrait(s) when needed) and `impl`ementing them as needed for different object kinds. On the other hand such approach (which seems to me as more corresponding to the idea of `trasit`s itself) have it's own overhead which turns out to be significant given 1) small/simple code that we're dealing with here, 2) demands deeper refactoring and consequent more changes to existing codebase.

Is this solution reasonable, or would it better to establish `trait` system and hide it a `mod` to minimize impact to the `lib.rs` file which was given, or there a better approach that I just don't see?

```diff
 
 fn get_distance(x1: i32, y1: i32, x2: i32, y2: i32) -> i32 {
     (((x1 - x2) * (x1 - x2) + (y1 - y2) * (y1 - y2)) as f64).sqrt() as i32
 }
 
-fn apply_physics(mut objects: Vec<ObjectType>, gravitational_constant: i32) -> Vec<ObjectType> {
+fn apply_physics(mut objects: Vec<Box<dyn Object>>, gravitational_constant: i32) -> Vec<Box<dyn Object>> {
     // Go through each pair of objects, and apply
     let gravity_sources = objects
         .iter()
         .filter_map(|o| {
-            return if let ObjectType::Planet(p) = o {
-                Some((p.coordinate.clone(), p.weight))
+            return if o.is_gravity_source() {
+                Some((o.get_coordinate().clone(), o.weight().expect(BREAKING_VALUE_KIND)))
             } else {
                 None
             };
@@ -110,55 +89,82 @@ fn apply_physics(mut objects: Vec<ObjectType>, gravitational_constant: i32) -> V
         .collect::<Vec<_>>();
 
     objects.iter_mut().for_each(|o| {
-        if let ObjectType::Asteroid(asteroid) = o {
+        if o.is_gravity_receiver() {
+            let asteroid = o;
             gravity_sources
                 .iter()
                 .for_each(|(planet_coord, planet_weight)| {
                     let distance = get_distance(
                         planet_coord.x,
                         planet_coord.y,
-                        asteroid.coordinate.x,
-                        asteroid.coordinate.y,
+                        asteroid.coordinate().x,
+                        asteroid.coordinate().y,
                     );
                     let distance = distance * distance;
 
                     let force = Direction {
-                        x: (asteroid.coordinate.x - planet_coord.x)
+                        x: (asteroid.coordinate().x - planet_coord.x)
                             * planet_weight
                             * gravitational_constant
                             / distance,
-                        y: (asteroid.coordinate.y - planet_coord.y)
+                        y: (asteroid.coordinate().y - planet_coord.y)
                             * planet_weight
                             * gravitational_constant
                             / distance,
                     };
-                    asteroid.velocity.x -= force.x;
-                    asteroid.velocity.y -= force.y;
+                    asteroid.velocity().expect(BREAKING_VALUE_KIND).x -= force.x;
+                    asteroid.velocity().expect(BREAKING_VALUE_KIND).y -= force.y;
 
-                    let vel = asteroid.velocity.clone();
+                    let vel = asteroid.velocity().expect(BREAKING_VALUE_KIND).clone();
                 })
         }
     });
 
     // Apply the new velocity to each object.
     objects.iter_mut().for_each(|object| {
-        if let ObjectType::Asteroid(asteroid) = object {
-            asteroid.coordinate.x += asteroid.velocity.x;
-            asteroid.coordinate.y += asteroid.velocity.y;
+        if object.is_gravity_receiver() {
+            object.coordinate().x += object.velocity().expect(BREAKING_VALUE_KIND).x;
+            object.coordinate().y += object.velocity().expect(BREAKING_VALUE_KIND).y;
         }
     });
 
     objects
```
These all is plain replacing the `enum` with `Box`ed trait object, needed fixes *and* insertion of `struct Circle` mentioned earlier.
```diff
 }
 
 fn handle_connection(
     mut stream: TcpStream,
-    mut objects: Vec<ObjectType>,
+    mut objects: Vec<Box<dyn Object>>,
     gravitational_constant: i32,
-) -> Vec<ObjectType> {
+) -> Vec<Box<dyn Object>> {
     objects = apply_physics(objects, gravitational_constant);
 
-    let circles = objects.iter().map(|o| o.get_circle()).collect::<Vec<_>>();
+    #[derive(Deserialize, Serialize)]
+    struct Circle {
+        cx: i32,
+        cy: i32,
+        r: i32,
+        stroke: String,
+        fill: String,
+        #[serde(rename = "stroke-width")]
+        stroke_width: i32,
+    }
+
+    let get_circle = |o: &Box<dyn Object>| -> Circle {
+        match (o.is_gravity_source(), o.is_gravity_receiver()) {
+            (true, false) => Circle { 
+                cx: o.get_coordinate().x, cy: o.get_coordinate().y, r: o.weight().expect(BREAKING_VALUE_KIND), stroke: "green".to_string(), 
+                fill: "black".to_string(), stroke_width: 3
+            },
+            (false, true) => Circle { 
+                cx: o.get_coordinate().x, cy: o.get_coordinate().y, r: 2, stroke: "green".to_string(
), 
+                fill: "black".to_string(), stroke_width: 3
+            },
+            (true, true) => todo!(),
+            (false, false) => todo!(),
+        }
+    };
+    let circles = objects.iter().map(|o| get_circle(o)).collect::<Vec<_>>();
     let contents = serde_json::to_string(&circles).unwrap();
     let status_line = "HTTP/1.1 200 OK";
     let response = format!(
@@ -168,10 +174,10 @@ fn handle_connection(
     stream.flush().unwrap();
     stream.shutdown(std::net::Shutdown::Both).unwrap();
 
```
```diff
     objects
 }
 
-pub fn start_server(uri: &str, mut objects: Vec<ObjectType>, gravitational_constant: i32) -> ! {
+pub fn start_server(uri: &str, mut objects: Vec<Box<dyn Object>>, gravitational_constant: i32) -> ! {
     let listener = TcpListener::bind(uri).unwrap();
 
     for stream in listener.incoming() {
diff --git a/src/main.rs b/src/main.rs
index b165e1f..03ad9ff 100644
--- a/src/main.rs
+++ b/src/main.rs
@@ -1,16 +1,16 @@
 use simulator_lib::directions::{coordinate::Coordinate, direction::Direction};
-use simulator_lib::{start_server, Asteroid, ObjectType, Planet};
+use simulator_lib::{start_server, Asteroid, Object, Planet};
 fn main() {
-    let mut objects = vec![
-        ObjectType::Planet(Planet {
+    let mut objects: Vec<Box<dyn Object>> = vec![
+        Box::new(Planet {
             coordinate: Coordinate::new(500, 500),
             weight: 50,
         }),
-        ObjectType::Asteroid(Asteroid {
+        Box::new(Asteroid {
             coordinate: Coordinate::new(250, 250),
             velocity: Direction { x: 30, y: -40 },
         }),
-        ObjectType::Asteroid(Asteroid {
+        Box::new(Asteroid {
             coordinate: Coordinate::new(750, 750),
             velocity: Direction { x: -30, y: 40 },
         }),
```

## Exercise:
## Workshop 5 - So.. Physics. Physics, Eh? Physics physics physics physics...
> ...

> Your tutor will discuss what polymorphism is, and the three approaches to polymorphism in Rust:
> 
>     Enums
    Generics
    Dynamic Dispatch
> 
> You should be able to identify the broad ideas of how they work, and the reasons you might choose one over the other.

### The Workshop
This week's workshop will explore Rust's trait system in more detail. You have been provided with code that does not use traits at all, which simulates the motion of bodies under gravity. Your task will be to gradually refactor the code towards using traits, rather than an enum.

#### Task 1: Understanding The Code You Have Been Provided

In the library you have been provided, there are two types of object defined in an enum. `Planets` do not move, but apply gravity to other objects. `Asteroids` move with a certain initial velocity, and are affected by gravity.

Since the enum is defined by the library, it is not possible to extend the library's behaviour to include different object types. In this task, you will modify the code so that it can support user-defined objects.

#### Task 2: Starting the simulator
In your starter code, you have been provided an HTML file called "phys_simulator.html". Open this file in your web browser to see a simulation of planets orbiting a star on your screen. You will see the small dots (asteroids), orbiting the large dot (planet).

#### Task 3: Removing the Enum

In the current code, you've been provided the `ObjectType` enum. As a user of the library, this gives you very little flexibility on what you can simulate: you are limited to asteroids and planets. In this workshop, we will be making the library more flexible, such that a user could implement their own types of celestial objects.

For the moment, we'll be changing our code so we can model objects which are affected by gravity, and which provide gravity. By the end of the tutorial, we'll be able to model any object that does both, but for now this allows us to make small changes to our code in each step.

Therefore, refactor the code so that rather than taking a vec of enums, it takes a vec of Planets, and a vec of Asteroids. Once you are done, you should be able to entirely remove the `ObjectType` enum.

#### Task 4: Defining Shared Behaviour

You'll notice that both Planets and Asteroids share code which defines their position, and converts them into a Circle struct to be sent to the front-end. This is shared behaviour which we can use a trait to represent.

Refactor the code so that Planets and Asteroids share a trait which defines their position, and allows conversion into a Circle. 

#### ...
Other tasks and solutions will be provided on another page.

# PS 
AFAIU the only thing that shouldn't be publicly shared are solutions to the graded exercises/activities, which I keep access restricted. If I got anything wrong and this code shouldn't be public as well, pls approach me by any mean you like, and I'll remove it without hesitations. Tom received couple of my contacts (`macrokata`/`lifetimekata` repos, <https://discord.com/channels/1075940806004838470/1075940806004838475/1101303841531646092>).

# references
[workshop]: https://cgi.cse.unsw.edu.au/~cs6991/23T1/workshop/05/questions
[one other]: https://codereview.stackexchange.com/questions/285476/reading-typed-data-from-file-over-ffi-with-libc