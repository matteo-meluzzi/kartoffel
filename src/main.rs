#![no_std]
#![no_main]

use kartoffel::*;
use threat_map::{coordinates::Coordinate, direction::Direction, greedy_next_move::greedy_next_move, orientation::Orientation, robot_position::RobotPosition, threat_map::ThreatMap};

struct Robot {
    threat_map: ThreatMap,
    enemy_positions: [Coordinate; threat_map::N * threat_map::N],
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
        self.robot_position.position = Coordinate::new(0, 0);
        let scan = radar_scan_9x9();
        // println!("{:?}", self.robot_position.orientation);
        // print_scan(&scan);

        let enemy_count = self.calculate_enemy_positions(&scan);
        // println!("{enemy_count} enemies detected");
        self.enemy_positions[0..enemy_count].iter_mut().for_each(|position| *position = position.orientate_north(self.robot_position.orientation));
        self.threat_map.calculate(&self.enemy_positions[0..enemy_count]);

        for y in -4..=4 {
            for x in -4..=4 {
                if scan.at(x, y) == ' ' {
                    self.threat_map.mask_border(Coordinate::new(x, y).orientate_north(self.robot_position.orientation));
                }
            }
        }

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

    fn calculate_enemy_positions(&mut self, scan: &RadarScan<9>) -> usize {
        let mut enemy_count = 0;
        for y in -4..=4 {
            for x in -4..=4 {
                if x == 0 && y == 0 {
                    // we are not an enemy
                    continue;
                }
                if scan.at(x, y) == '@' {
                    self.enemy_positions[enemy_count] = Coordinate{x, y};
                    enemy_count += 1;
                }
            }
        }
        enemy_count
    }
}

#[no_mangle]
fn main() {
    let orientation = Orientation::from_integer(compass_dir() as i32 - 1).unwrap();
    let mut robot = Robot{ threat_map: ThreatMap::new(), enemy_positions: [Coordinate::new(0, 0); 9 * 9], robot_position: RobotPosition {position: Coordinate::new(0,0), orientation} };
    loop {
        robot.round_robin_step();
    }
}
