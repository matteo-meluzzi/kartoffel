#![no_std]
#![no_main]

use kartoffel::*;
use threat_map::{coordinates::Coordinate, direction::Direction, enemy_position::{EnemyPosition, EnemyPositions}, greedy_next_move::greedy_next_move, orientation::Orientation, robot_position::RobotPosition, threat_map::ThreatMap};

struct Robot {
    threat_map: ThreatMap,
    enemy_positions: EnemyPositions,
    previous_enemy_positions: EnemyPositions,
    robot_position: RobotPosition,
}

fn print_scan<const N:usize>(scan: &RadarScan<N>) {
    let n = N as i8;
    for y in -n/2..=n/2 {
        for x in -n/2..=n/2 {
            let c = scan.at(x, y);
            print!("{c}");
        }
        println!("");
    }
    println!("");
}

fn print_map<const N:usize>(map: &ThreatMap) {
    let n = N as i8;
    for y in -n/2..=n/2 {
        for x in -n/2..=n/2 {
            let c = map.at(Coordinate { x, y }).min(9);
            print!("{c}");
        }
        println!("");
    }
    println!("");
}

impl Robot {
    fn radar_is_ready(&mut self) {
        let scan = radar_scan_9x9();
        // println!("{:?}", self.robot_position.orientation);
        // print_scan(&scan);

        self.previous_enemy_positions = self.enemy_positions.clone();
        self.previous_enemy_positions.use_origin(self.robot_position.position);

        self.calculate_enemy_positions(&scan);
        
        self.threat_map.calculate_with_previous_location(&self.enemy_positions, &self.previous_enemy_positions);

        let n = threat_map::N as i8;
        for y in -n/2..=n/2 {
            for x in -n/2..=n/2 {
                if scan.at(x, y) == ' ' {
                    self.threat_map.mask_border(Coordinate::new(x, y).orientate_north(self.robot_position.orientation));
                }
            }
        }

        self.robot_position.position = Coordinate::new(0, 0);
        // print_map::<9>(&self.threat_map);
    }

    fn motor_is_ready(&mut self) {
        if let Some(step_direction) = greedy_next_move(&self.threat_map, &self.robot_position) {
            self.robot_position.take_step(step_direction);
            match step_direction {
                Direction::Front => {
                    motor_step_fw()
                }
                Direction::Back => {
                    motor_step_bw()
                }
                Direction::Right => {
                    motor_turn_right();
                }
                Direction::Left => {
                    motor_turn_left();
                }
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

    fn calculate_enemy_positions(&mut self, scan: &RadarScan<9>) {
        self.enemy_positions.clear();

        let n = threat_map::N as i8;
        for y in -n/2..=n/2 {
            for x in -n/2..=n/2 {
                    if x == 0 && y == 0 {
                    // we are not an enemy
                    continue;
                }
                if scan.at(x, y) == '@' {
                    let id = scan.bot_at(x, y).unwrap();
                    self.enemy_positions.push(EnemyPosition::new(id, Coordinate::new(x, y).orientate_north(self.robot_position.orientation)));
                }
            }
        }
    }
}

#[no_mangle]
fn main() {
    let orientation = Orientation::from_integer(compass_dir() as i32 - 1).unwrap();
    let mut robot = Robot{ threat_map: ThreatMap::new(), enemy_positions: EnemyPositions::new(), previous_enemy_positions: EnemyPositions::new(), robot_position: RobotPosition {position: Coordinate::new(0,0), orientation} };
    loop {
        robot.round_robin_step();
    }
}
