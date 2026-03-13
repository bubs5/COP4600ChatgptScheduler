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
/* TO BE ADDED */

fn first_in_first_out(_control: [i32;4], _processes: &Vec<Process>) -> ScheduleResult {
    let runfor = _control[1];

    // Local structure to track process state without modifying the original input
    #[derive(Clone)]
    struct ProcessTracker {
        name: String,
        arrival: i32,
        burst: i32,
        remaining: i32,
        started: bool,
        start_time: i32,
        finish_time: i32,
        finished: bool,
    }

    // Initialize tracking variables preserving the file order
    let mut trackers: Vec<ProcessTracker> = _processes.iter().map(|p| ProcessTracker {
        name: p.name.clone(),
        arrival: p.arrival,
        burst: p.burst,
        remaining: p.burst,
        started: false,
        start_time: 0,
        finish_time: 0,
        finished: false,
    }).collect();

    let mut events: Vec<Event> = Vec::new();
    let mut ready_queue: Vec<usize> = Vec::new();
    let mut running: Option<usize> = None;

    for t in 0..runfor {
        // 1. Check arrivals: Any process arriving at tick 't' is added to the ready queue.
        for (i, p) in trackers.iter().enumerate() {
            if p.arrival == t {
                events.push(Event { 
                    time: t, 
                    message: format!("{} arrived", p.name) 
                });
                ready_queue.push(i);
            }
        }

        // 2. Check completions: If the currently running process has finished its burst.
        if let Some(idx) = running {
            if trackers[idx].remaining == 0 {
                events.push(Event { 
                    time: t, 
                    message: format!("{} finished", trackers[idx].name) 
                });
                trackers[idx].finished = true;
                trackers[idx].finish_time = t;
                running = None;
            }
        }

        // 3. Process Selection: If the CPU is free, pick the next process in the ready queue.
        if running.is_none() {
            if !ready_queue.is_empty() {
                let idx = ready_queue.remove(0); // FCFS: remove the oldest arrival
                running = Some(idx);
                if !trackers[idx].started {
                    trackers[idx].started = true;
                    trackers[idx].start_time = t;
                }
                events.push(Event {
                    time: t,
                    message: format!("{} selected (burst {:>3})", trackers[idx].name, trackers[idx].burst)
                });
            }
        }

        // 4. Execution / CPU Idle State
        if let Some(idx) = running {
            trackers[idx].remaining -= 1;
        } else {
            events.push(Event { 
                time: t, 
                message: "Idle".to_string() 
            });
        }
    }

    // Post-loop check to ensure we catch any processes that finish on the very last tick
    if let Some(idx) = running {
        if trackers[idx].remaining == 0 {
            events.push(Event { 
                time: runfor, 
                message: format!("{} finished", trackers[idx].name) 
            });
            trackers[idx].finished = true;
            trackers[idx].finish_time = runfor;
        }
    }

    // Aggregate statistics. Only evaluate wait/turnaround logic on processes that actually finished.
    let mut stats: Vec<ProcessStats> = Vec::new();
    for p in trackers {
        if p.finished {
            let turnaround = p.finish_time - p.arrival;
            let wait = turnaround - p.burst;
            let response = p.start_time - p.arrival;

            stats.push(ProcessStats {
                name: p.name,
                wait,
                turnaround,
                response,
            });
        }
    }

    ScheduleResult {
        events,
        stats,
        finish_time: runfor,
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


