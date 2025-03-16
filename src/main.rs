#![no_std]
#![no_main]

mod kartoffel_nn;

use core::mem::swap;
use kartoffel::*;
use threat_map::{borders::Borders, coordinates::Coordinate, direction::Direction, enemy_position::{EnemyPosition, EnemyPositions}, orientation::Orientation, robot_position::RobotPosition, N};
use kartoffel_nn::{KARTOFFEL_NN, Fix};

fn argmax<T: Ord>(xs: &[T]) -> Option<usize> {
    xs.iter().enumerate().max_by_key(|(_, t)| *t).map(|(i, _)| i)
}

struct Robot {
    enemy_positions: EnemyPositions,
    previous_enemy_positions: EnemyPositions,
    robot_position: RobotPosition,
    borders: Borders
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

impl Robot {
    fn radar_is_ready(&mut self) {
        let scan = radar_scan_9x9();
        // println!("{:?}", self.robot_position.orientation);
        print_scan(&scan);

        // the current positions become old
        swap(&mut self.previous_enemy_positions, &mut self.enemy_positions);

        // ajdust coordinate system of previous enemies
        self.previous_enemy_positions.use_origin(self.robot_position.position);

        // get the new positions
        (self.enemy_positions, self.borders) = self.analyze_scan(&scan);
        
        // we are now at the center of the coordinate system again
        self.robot_position.position = Coordinate::new(0, 0);
    }

    fn get_observations(&self) -> [Fix; 244] {
        let mut observations = [Fix::ZERO; 244];

        for enemy_pos in &self.enemy_positions {
            let observations = &mut observations[..N*N];
            let index = enemy_pos.position.to_index().expect("current enemy position is out of the scan");
            observations[index] = Fix::ONE;
        }

        for prev_enemy_pos in &self.previous_enemy_positions {
            let observations = &mut observations[N*N..N*N*2];
            if let Some(index) = prev_enemy_pos.position.to_index() {
                observations[index] = Fix::ONE;
            }
        }

        let border_observations = &mut observations[N*N*2..N*N*3];
        let mut border_observations_index = 0;
        for y in 0..N as i8 {
            for x in 0..N as i8 {
                let coord = Coordinate::new(x, y);
                if self.borders.is_border(coord) {
                    border_observations[border_observations_index] = Fix::ONE;
                }
                border_observations_index += 1;
            }
        }

        observations[observations.len() - 1] = if is_arm_ready() { Fix::ZERO } else { Fix::ONE };

        observations
    }

    fn motor_is_ready(&mut self, step_direction: Direction) {
        self.robot_position.take_step(step_direction);
        match step_direction {
            Direction::Front => motor_step_fw(),
            Direction::Back => motor_step_bw(),
            Direction::Right => motor_turn_right(),
            Direction::Left => motor_turn_left()
        }
    }

    fn step(&mut self) {    
        radar_wait();
        self.radar_is_ready();
        let observations = self.get_observations();
        let nn_output = KARTOFFEL_NN.forward(observations);
        let nn_move = argmax(&nn_output).expect("nn output is empty");
        println!("nn move: {}", nn_move);
        match nn_move {
            nn_move@0..=3 => {
                let step_direction = match nn_move {
                    0 => Direction::Front,
                    1 => Direction::Back,
                    2 => Direction::Left,
                    3 => Direction::Right,
                    _ => unreachable!()
                };
                motor_wait();
                self.motor_is_ready(step_direction);
            }
            4 => {
                arm_wait();
                arm_stab();
            }
            _ => unreachable!()
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
        Robot{ enemy_positions: EnemyPositions::new(), previous_enemy_positions: EnemyPositions::new(), robot_position: RobotPosition {position: Coordinate::new(0,0), orientation}, borders: Borders::new() }
    }
}

#[no_mangle]
fn main() {
    let orientation = Orientation::from_integer(compass_dir() as i32 - 1).unwrap();

    let mut robot = Robot::new(orientation);
    loop {
        robot.step();
    }
}
