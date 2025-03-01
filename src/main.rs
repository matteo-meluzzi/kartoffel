#![no_std]
#![no_main]

use core::mem::swap;

use kartoffel::*;
use threat_map::{borders::Borders, coordinates::Coordinate, direction::Direction, enemy_position::{EnemyPosition, EnemyPositions}, enemy_position_prediction::EnemyPositionPrediction, greedy_next_move::greedy_next_move, orientation::Orientation, robot_position::RobotPosition};

struct Robot {
    enemy_positions: EnemyPositions,
    previous_enemy_positions: EnemyPositions,
    enemy_position_prediction: EnemyPositionPrediction,
    robot_position: RobotPosition,
    borders: Borders
}

// fn print_scan<const N:usize>(scan: &RadarScan<N>) {
//     let n = N as i8;
//     for y in -n/2..=n/2 {
//         for x in -n/2..=n/2 {
//             let c = scan.at(x, y);
//             print!("{c}");
//         }
//         println!("");
//     }
//     println!("");
// }

// fn print_map<const N:usize>(map: &ThreatMap) {
//     let n = N as i8;
//     for y in -n/2..=n/2 {
//         for x in -n/2..=n/2 {
//             let c = map.at(Coordinate { x, y }).min(9);
//             print!("{c}");
//         }
//         println!("");
//     }
//     println!("");
// }

impl Robot {
    fn radar_is_ready(&mut self) {
        let scan = radar_scan_9x9();
        // println!("{:?}", self.robot_position.orientation);
        // print_scan(&scan);

        // the current positions become old
        swap(&mut self.previous_enemy_positions, &mut self.enemy_positions);

        // ajdust coordinate system of previous enemies
        self.previous_enemy_positions.use_origin(self.robot_position.position);

        // get the new positions
        (self.enemy_positions, self.borders) = self.analyze_scan(&scan);
        
        // update predictions
        self.enemy_position_prediction = EnemyPositionPrediction::new(&self.enemy_positions, &self.previous_enemy_positions, self.borders.clone());

        // we are now at the center of the coordinate system again
        self.robot_position.position = Coordinate::new(0, 0);
    }

    fn motor_is_ready(&mut self) {
        // assume enemies are one step ahead
        self.enemy_position_prediction.move_enemies(); 

        if let Some(step_direction) = greedy_next_move(&self.robot_position, &self.enemy_position_prediction, &self.borders) {
            self.robot_position.take_step(step_direction);
            match step_direction {
                Direction::Front => motor_step_fw(),
                Direction::Back => motor_step_bw(),
                Direction::Right => motor_turn_right(),
                Direction::Left => motor_turn_left()
            }
        }
    }

    fn round_robin_step(&mut self) {    
        if is_radar_ready() {
            self.radar_is_ready();
        } 

        if is_motor_ready() {
            self.motor_is_ready();
        }
    }

    fn analyze_scan(&mut self, scan: &RadarScan<9>) -> (EnemyPositions, Borders) {
        let mut enemy_positions = EnemyPositions::new();
        let mut borders = Borders::new();

        let n = threat_map::N as i8;
        for y in -n/2..=n/2 {
            for x in -n/2..=n/2 {
                    if x == 0 && y == 0 {
                    // we are not an enemy
                    continue;
                }
                let c = scan.at(x, y);
                let coord = Coordinate::new(x, y).orientate_north(self.robot_position.orientation);
                if c == '@' {
                    let id = scan.bot_at(x, y).unwrap();
                    enemy_positions.push(EnemyPosition::new(id, coord));
                }
                if c == ' ' {
                    borders.set_border(coord);
                }
            }
        }

        (enemy_positions, borders)
    }

    fn new(orientation: Orientation) -> Self {
        Robot{ enemy_positions: EnemyPositions::new(), previous_enemy_positions: EnemyPositions::new(), robot_position: RobotPosition {position: Coordinate::new(0,0), orientation}, enemy_position_prediction: EnemyPositionPrediction::empty(), borders: Borders::new() }
    }
}

#[no_mangle]
fn main() {
    let orientation = Orientation::from_integer(compass_dir() as i32 - 1).unwrap();

    let mut robot = Robot::new(orientation);
    loop {
        robot.round_robin_step();
    }
}
