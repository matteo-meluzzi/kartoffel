#![no_std]
#![no_main]

use kartoffel::*;
use threat_map::{coordinates::BotCentricCoordinate, greedy_next_move::greedy_next_move, threat_map::ThreatMap, Move};

struct Robot {
    threat_map: ThreatMap,
    enemy_positions: [BotCentricCoordinate; 9 * 9]
}

impl Robot {
    fn round_robin_step(&mut self) {    
        if is_radar_ready() {
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
            self.threat_map.calculate(&self.enemy_positions[0..enemy_count]);
            self.threat_map.mask_border(&|coord| { scan.at(coord.x, coord.y) == ' ' });
        } 

        if is_motor_ready() {
            if let Some(mov) = greedy_next_move(&self.threat_map) {
                // println!("{:?}", mov);
                match mov {
                    Move::Front => motor_step_fw(),
                    Move::Back => motor_step_bw(),
                    Move::Right => {
                        motor_turn_right();
                        motor_wait();
                        motor_step_fw();
                    }
                    Move::Left => {
                        motor_turn_left();
                        motor_wait();
                        motor_step_fw();
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
                    self.enemy_positions[enemy_count] = BotCentricCoordinate{x, y};
                    enemy_count += 1;
                }
            }
        }
        enemy_count
    }
}

#[no_mangle]
fn main() {
    let mut robot = Robot{ threat_map: ThreatMap::new(), enemy_positions: [BotCentricCoordinate::new(0, 0); 9 * 9] };
    loop {
        robot.round_robin_step();
    }
}
