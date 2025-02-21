#![no_std]
#![no_main]

use kartoffel::*;
use threat_map::{coordinates::Coordinate, greedy_next_move::greedy_next_move, orientation::Orientation, threat_map::ThreatMap, direction::Direction};

struct Robot {
    threat_map: ThreatMap,
    position_in_threat_map: Coordinate,
    enemy_positions: [Coordinate; 9 * 9],
    orientation: Orientation
}

impl Robot {
    fn round_robin_step(&mut self) {    
        if is_radar_ready() {
            self.position_in_threat_map = Coordinate::new(0, 0);
            let scan = radar_scan_9x9();
            // for y in -4..=4 {
            //     for x in -4..=4 {
            //         let c = scan.at(x, y);
            //         print!("{c}");
            //     }
            //     println!("");
            // }
            // println!("");

            let enemy_count = self.calculate_enemy_positions(&scan);
            self.threat_map.update_orientation(self.orientation);
            self.threat_map.calculate(&self.enemy_positions[0..enemy_count]);
            self.threat_map.mask_border(&|coord| { scan.at(coord.x, coord.y) == ' ' });
        } 

        if is_motor_ready() {
            if let Some(mov) = greedy_next_move(&self.threat_map, self.orientation, self.position_in_threat_map) {
                match mov {
                    Direction::Front => {
                        self.position_in_threat_map = self.position_in_threat_map.in_direction(self.orientation.relative_to(self.threat_map.orientation));
                        motor_step_fw()
                    }
                    Direction::Back => {
                        self.position_in_threat_map = self.position_in_threat_map.in_direction(self.orientation.rotated_right().rotated_right().relative_to(self.threat_map.orientation));
                        motor_step_bw()
                    }
                    Direction::Right => {
                        motor_turn_right();
                        self.orientation = self.orientation.rotated_right()
                    }
                    Direction::Left => {
                        motor_turn_left();
                        self.orientation = self.orientation.rotated_left()
                    }
                }
            } else {
                // println!("Staying")
            }
        }
    }

    fn calculate_enemy_positions(&mut self, scan: &RadarScan<9>) -> usize {
        let mut enemy_count = 0;
        for y in -4..=4 {
            for x in -4..4 {
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
    let mut robot = Robot{ threat_map: ThreatMap::new(orientation), enemy_positions: [Coordinate::new(0, 0); 9 * 9], orientation, position_in_threat_map: Coordinate::new(0, 0) };
    loop {
        robot.round_robin_step();
    }
}
