#![no_std]
#![no_main]

mod kartoffel_nn;

use kartoffel::*;
use kartoffel_nn::{KARTOFFEL_NN, Fix};

const N: usize = 7;

fn argmax<T: Ord>(xs: &[T]) -> Option<usize> {
    xs.iter().enumerate().max_by_key(|(_, t)| *t).map(|(i, _)| i)
}

struct Robot {
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
    fn get_observations(&self, scan: &RadarScan<N>) -> [Fix; 50] {
        let mut observations = [Fix::ZERO; 50];

        let n = threat_map::N as i8;
        for i in 0..49 {
            let x = i%n - n/2;
            let y = i/n - n/2;
            let index = i as usize;

            let c = scan.at(x, y);
            if c == ' ' || c == '@' {
                observations[index] = Fix::ONE;
            }
        }

        observations[observations.len() - 1] = if is_arm_ready() { Fix::ZERO } else { Fix::ONE };

        // for (i, o) in observations.iter().enumerate() {
        //     print!("{o} ");
        //     if i % N == N - 1 {
        //         println!("");
        //     }
        // }
        // println!("");
    
        observations
    }

    fn step(&mut self) {    
        radar_wait();
        let scan = radar_scan_7x7();
        // print_scan(&scan);
        let observations = self.get_observations(&scan);
        let nn_output = KARTOFFEL_NN.forward(observations);
        let nn_move = argmax(&nn_output).expect("nn output is empty");
        // println!("nn move: {nn_move}");
        match nn_move {
            nn_move@0..=3 => {
                motor_wait();
                match nn_move {
                    0 => motor_step_fw(),
                    1 => motor_step_bw(),
                    2 => motor_turn_left(),
                    3 => motor_turn_right(),
                    _ => unreachable!()
                };
            }
            4 => {
                arm_wait();
                arm_stab();
            }
            5 => (),
            _ => unreachable!()
        }
    }

    fn new() -> Self {
        Robot{ }
    }
}

#[no_mangle]
fn main() {
    let mut robot = Robot::new();
    loop {
        robot.step();
    }
}
