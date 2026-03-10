use std::env;
use std::fs::{read_to_string, File};
use std::io::Write;
use std::process;

/* ---------------- PROCESS INPUT STRUCT ---------------- */

#[derive(Clone)]
struct Process {
    name: String,
    arrival: i32,
    burst: i32,
}

/* ---------------- OUTPUT STRUCTS ---------------- */

struct Event {
    time: i32,
    message: String,
}

struct ProcessStats {
    name: String,
    wait: i32,
    turnaround: i32,
    response: i32,
}

struct ScheduleResult {
    events: Vec<Event>,
    stats: Vec<ProcessStats>,
    finish_time: i32,
}

/* ---------------- MAIN ---------------- */

fn main() {

    /* ---------- READ INPUT FILE ---------- */

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Error: input file required");
        process::exit(1);
    }

    let filename = &args[1];

    let contents = match read_to_string(filename) {
        Ok(c) => c,
        Err(_) => {
            eprintln!("Error: cannot read input file");
            process::exit(1);
        }
    };

    /* ---------- CONTROL MATRIX ---------- */

    let mut processcount: i32 = 0;
    let mut runfor: i32 = 0;
    let mut use_alg: i32 = -1;
    let mut quantum: i32 = 0;

    /* ---------- PROCESS MATRIX ---------- */

    let mut processes: Vec<Process> = Vec::new();

    /* ---------- PARSE INPUT ---------- */

    for line in contents.lines() {

        let tokens: Vec<&str> = line.split_whitespace().collect();

        if tokens.is_empty() {
            continue;
        }

        match tokens[0] {

            "processcount" => {
                processcount = tokens[1].parse().unwrap();
            }

            "runfor" => {
                runfor = tokens[1].parse().unwrap();
            }

            "use" => {
                use_alg = match tokens[1] {
                    "fcfs" => 0,
                    "sjf" => 1,
                    "rr" => 2,
                    _ => {
                        eprintln!("Error: invalid scheduling algorithm");
                        process::exit(1);
                    }
                };
            }

            "quantum" => {
                quantum = tokens[1].parse().unwrap();
            }

            "process" => {

                let name = tokens[2].to_string();
                let arrival: i32 = tokens[4].parse().unwrap();
                let burst: i32 = tokens[6].parse().unwrap();

                processes.push(Process {
                    name,
                    arrival,
                    burst,
                });
            }

            "end" => break,

            _ => {}
        }
    }

    /* ---------- RR QUANTUM VALIDATION ---------- */

    if use_alg == 2 && quantum == 0 {
        eprintln!("Error: Round Robin requires a quantum");
        process::exit(1);
    }

    /* ---------- CONTROL MATRIX ---------- */

    let control_matrix = [processcount, runfor, use_alg, quantum];

    /* ---------- CALL SCHEDULER ---------- */

    let result = match use_alg {
        0 => first_in_first_out(control_matrix, &processes),
        1 => shortest_job_first(control_matrix, &processes),
        2 => round_robin(control_matrix, &processes),
        _ => {
            eprintln!("Error: invalid algorithm");
            process::exit(1);
        }
    };

    /* ---------- CREATE OUTPUT FILE ---------- */

    let output_name = filename.trim_end_matches(".in").to_string() + ".out";
    let mut file = File::create(output_name).unwrap();

    /* ---------- HEADER ---------- */

    writeln!(file, "{} processes", processcount).unwrap();

    match use_alg {

        0 => {
            writeln!(file, "Using First-Come First-Served").unwrap();
        }

        1 => {
            writeln!(file, "Using preemptive Shortest Job First").unwrap();
        }

        2 => {
            writeln!(file, "Using Round-Robin").unwrap();
            writeln!(file, "Quantum {}", quantum).unwrap();
        }

        _ => {}
    }

    writeln!(file).unwrap();

    /* ---------- EVENT LOG ---------- */

    for event in result.events {

        writeln!(
            file,
            "Time {:>3} : {}",
            event.time,
            event.message
        ).unwrap();
    }

    /* ---------- FINISH TIME ---------- */

    writeln!(
        file,
        "Finished at time {:>3}",
        result.finish_time
    ).unwrap();

    writeln!(file).unwrap();

    /* ---------- PROCESS STATS ---------- */

    for stat in result.stats {

        writeln!(
            file,
            "{} wait {:>3} turnaround {:>3} response {:>3}",
            stat.name,
            stat.wait,
            stat.turnaround,
            stat.response
        ).unwrap();
    }
}

/* ---------------- SCHEDULING FUNCTIONS ---------------- */
/* DO NOT IMPLEMENT IN THIS FILE (as requested) */

fn first_in_first_out(_control: [i32;4], _processes: &Vec<Process>) -> ScheduleResult {

    ScheduleResult {
        events: vec![],
        stats: vec![],
        finish_time: 0,
    }
}

fn shortest_job_first(_control: [i32;4], _processes: &Vec<Process>) -> ScheduleResult {

    ScheduleResult {
        events: vec![],
        stats: vec![],
        finish_time: 0,
    }
}

fn round_robin(_control: [i32;4], _processes: &Vec<Process>) -> ScheduleResult {

    ScheduleResult {
        events: vec![],
        stats: vec![],
        finish_time: 0,
    }

}
