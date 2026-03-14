/*
Brandon Saavedra
Caleb Berent
Carson Prewitt
Isaac Rucker
*/
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

    // Check for flags and filename
    let has_graph = args.iter().any(|arg| arg == "--graph");
    let filename = args.iter().find(|arg| arg.ends_with(".in"));

    if filename.is_none() {
        eprintln!("Usage: scheduler-gpt <input file> [--graph]");
        process::exit(1);
    }

    let filename = filename.unwrap();

    let contents = match read_to_string(filename) {
        Ok(c) => c,
        Err(_) => {
            eprintln!("Error: cannot read input file");
            process::exit(1);
        }
    };

    /* ---------- CONTROL MATRIX ---------- */

    let mut processcount: Option<i32> = None;
    let mut runfor: Option<i32> = None;
    let mut use_alg: Option<i32> = None;
    let mut quantum: Option<i32> = None;

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
                if tokens.len() < 2 {
                    eprintln!("Error: Missing parameter processcount");
                    process::exit(1);
                }
                processcount = Some(match tokens[1].parse() {
                    Ok(v) => v,
                    Err(_) => {
                        eprintln!("Error: Bad data type for processcount");
                        process::exit(1);
                    }
                });
            }

            "runfor" => {
                if tokens.len() < 2 {
                    eprintln!("Error: Missing parameter runfor");
                    process::exit(1);
                }
                runfor = Some(match tokens[1].parse() {
                    Ok(v) => v,
                    Err(_) => {
                        eprintln!("Error: Bad data type for runfor");
                        process::exit(1);
                    }
                });
            }

            "use" => {
                if tokens.len() < 2 {
                    eprintln!("Error: Missing parameter use");
                    process::exit(1);
                }
                use_alg = Some(match tokens[1] {
                    "fcfs" => 0,
                    "sjf" => 1,
                    "rr" => 2,
                    _ => {
                        eprintln!("Error: invalid scheduling algorithm");
                        process::exit(1);
                    }
                });
            }

            "quantum" => {
                if tokens.len() < 2 {
                    eprintln!("Error: Missing parameter quantum");
                    process::exit(1);
                }
                quantum = Some(match tokens[1].parse() {
                    Ok(v) => v,
                    Err(_) => {
                        eprintln!("Error: Bad data type for quantum");
                        process::exit(1);
                    }
                });
            }

            "process" => {
                if tokens.len() < 7 {
                    eprintln!("Error: Missing parameter in process definition");
                    process::exit(1);
                }
                let name = tokens[2].to_string();
                let arrival: i32 = match tokens[4].parse() {
                    Ok(v) => v,
                    Err(_) => {
                        eprintln!("Error: Bad data type for arrival");
                        process::exit(1);
                    }
                };
                let burst: i32 = match tokens[6].parse() {
                    Ok(v) => v,
                    Err(_) => {
                        eprintln!("Error: Bad data type for burst");
                        process::exit(1);
                    }
                };

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

    /* ---------- MISSING PARAMETER VALIDATION ---------- */

    if processcount.is_none() {
        eprintln!("Error: Missing parameter processcount");
        process::exit(1);
    }
    if runfor.is_none() {
        eprintln!("Error: Missing parameter runfor");
        process::exit(1);
    }
    if use_alg.is_none() {
        eprintln!("Error: Missing parameter use");
        process::exit(1);
    }

    /* ---------- RR QUANTUM VALIDATION ---------- */

    if use_alg == Some(2) && quantum.is_none() {
        eprintln!("Error: Missing quantum parameter when use is rr");
        process::exit(1);
    }

    /* ---------- CONTROL MATRIX ---------- */

    let processcount = processcount.unwrap();
    let runfor = runfor.unwrap();
    let use_alg = use_alg.unwrap();
    let quantum = quantum.unwrap_or(0);

    let control_matrix = [processcount, runfor, use_alg, quantum];

    /* ---------- CALL SCHEDULER ---------- */

    let result = match use_alg {
        0 => first_in_first_out(control_matrix, &processes),
        1 => shortest_job_first(control_matrix, &processes),
        2 => round_robin(control_matrix, &processes),
        3 => linux_cfs(control_matrix, &processes),
        _ => {
            eprintln!("Error: invalid algorithm");
            process::exit(1);
        }
    };

    /* ---------- CREATE OUTPUT FILE ---------- */

    let output_name = filename.trim_end_matches(".in").to_string() + ".out";
    let mut file = File::create(output_name).unwrap();

    /* ---------- HEADER ---------- */

    writeln!(file, "{:>3} processes", processcount).unwrap();

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

    for event in &result.events {

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

    for stat in &result.stats {

        writeln!(
            file,
            "{} wait {:>3} turnaround {:>3} response {:>3}",
            stat.name,
            stat.wait,
            stat.turnaround,
            stat.response
        ).unwrap();
    }

    // After calling your scheduler and getting 'result':
    if has_graph {
        print_htop_graph(&result, &processes, runfor);
    }
}

fn print_htop_graph(result: &ScheduleResult, processes: &Vec<Process>, run_for: i32) {
    // 1. Header Information
    println!("\x1b[1;37m  Tasks:\x1b[0m \x1b[1;32m{} total\x1b[0m", processes.len());
    println!("\x1b[1;37m  Finish Time:\x1b[0m \x1b[1;34m{}\x1b[0m", result.finish_time);

    // 2. CPU Timeline Bar (The htop "Memory/CPU" style)
    print!("\x1b[1;37m  CPU \x1b[0m[");
    
    let mut timeline = vec!["\x1b[1;30m.\x1b[0m".to_string(); run_for as usize];
    let colors = ["\x1b[42;30m", "\x1b[44;37m", "\x1b[45;37m", "\x1b[43;30m", "\x1b[46;30m"];
    
    use std::collections::HashMap;
    let mut color_map = HashMap::new();
    for (i, p) in processes.iter().enumerate() {
        color_map.insert(p.name.clone(), colors[i % colors.len()]);
    }

    let mut current_proc: Option<String> = None;
    for t in 0..run_for {
        for event in &result.events {
            if event.time == t {
                if event.message.contains("selected") {
                    current_proc = Some(event.message.split_whitespace().next().unwrap().to_string());
                } else if event.message.contains("finished") || event.message == "Idle" {
                    current_proc = None;
                }
            }
        }
        if let Some(ref name) = current_proc {
            let color = color_map.get(name).unwrap_or(&"\x1b[47;30m");
            timeline[t as usize] = format!("{}{}{}", color, name.chars().next().unwrap(), "\x1b[0m");
        }
    }

    for slot in timeline { print!("{}", slot); }
    println!("] \x1b[1;32m100.0%\x1b[0m");

    // 3. Process Table (htop "Process List" style)
    println!("\n\x1b[7;37m  NAME      WAIT   TURNAROUND   RESPONSE   BURST \x1b[0m");
    
    for (i, stat) in result.stats.iter().enumerate() {
        let p_info = &processes[i];
        let color_code = color_map.get(&stat.name).unwrap_or(&"\x1b[0m");
        
        // Print each row with slight indentation and aligned columns
        println!(
            "  {:<8}  {:>4}   {:>10}   {:>8}   {:>5}",
            format!("{}{} \x1b[0m", color_code, stat.name),
            stat.wait,
            stat.turnaround,
            stat.response,
            p_info.burst
        );
    }

    // 4. Footer Message for Unfinished Processes
    for event in &result.events {
        if event.message.contains("did not finish") {
            println!("\x1b[1;31m  ! {}\x1b[0m", event.message);
        }
    }
    println!();
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
    let runfor = _control[1];

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
        order: usize,
    }

    let mut trackers: Vec<ProcessTracker> = _processes
        .iter()
        .enumerate()
        .map(|(i, p)| ProcessTracker {
            name: p.name.clone(),
            arrival: p.arrival,
            burst: p.burst,
            remaining: p.burst,
            started: false,
            start_time: 0,
            finish_time: 0,
            finished: false,
            order: i,
        })
        .collect();

    let mut events: Vec<Event> = Vec::new();
    let mut running: Option<usize> = None;
    let mut last_selected: Option<usize> = None;

    for t in 0..runfor {
        // 1. Log arrivals first
        for p in trackers.iter() {
            if p.arrival == t {
                events.push(Event {
                    time: t,
                    message: format!("{} arrived", p.name),
                });
            }
        }

        // 2. If something finished at the end of the previous tick, log it now
        if let Some(idx) = running {
            if trackers[idx].remaining == 0 {
                trackers[idx].finished = true;
                trackers[idx].finish_time = t;
                events.push(Event {
                    time: t,
                    message: format!("{} finished", trackers[idx].name),
                });
                running = None;
                last_selected = None;
            }
        }

        // 3. Pick the ready process with shortest remaining time
        let mut best: Option<usize> = None;
        for i in 0..trackers.len() {
            if trackers[i].arrival <= t && !trackers[i].finished && trackers[i].remaining > 0 {
                match best {
                    None => best = Some(i),
                    Some(bi) => {
                        let better = trackers[i].remaining < trackers[bi].remaining
                            || (trackers[i].remaining == trackers[bi].remaining
                                && trackers[i].arrival < trackers[bi].arrival)
                            || (trackers[i].remaining == trackers[bi].remaining
                                && trackers[i].arrival == trackers[bi].arrival
                                && trackers[i].order < trackers[bi].order);

                        if better {
                            best = Some(i);
                        }
                    }
                }
            }
        }

        running = best;

        // 4. Run selected process or log Idle
        if let Some(idx) = running {
            if !trackers[idx].started {
                trackers[idx].started = true;
                trackers[idx].start_time = t;
            }

            if last_selected != Some(idx) {
                events.push(Event {
                    time: t,
                    message: format!(
                        "{} selected (burst {:>3})",
                        trackers[idx].name,
                        trackers[idx].remaining
                    ),
                });
                last_selected = Some(idx);
            }

            trackers[idx].remaining -= 1;
        } else {
            events.push(Event {
                time: t,
                message: "Idle".to_string(),
            });
            last_selected = None;
        }
    }

    // 5. Catch completion exactly at runfor
    if let Some(idx) = running {
        if trackers[idx].remaining == 0 {
            trackers[idx].finished = true;
            trackers[idx].finish_time = runfor;
            events.push(Event {
                time: runfor,
                message: format!("{} finished", trackers[idx].name),
            });
        }
    }

    // 6. Build stats in original input order
    let mut stats: Vec<ProcessStats> = Vec::new();
    for p in trackers.iter() {
        if p.finished {
            let turnaround = p.finish_time - p.arrival;
            let wait = turnaround - p.burst;
            let response = p.start_time - p.arrival;

            stats.push(ProcessStats {
                name: p.name.clone(),
                wait,
                turnaround,
                response,
            });
        } else {
            // Current output code doesn't support "did not finish",
            // so use sentinel values for unfinished processes.
            stats.push(ProcessStats {
                name: p.name.clone(),
                wait: -1,
                turnaround: -1,
                response: if p.started { p.start_time - p.arrival } else { -1 },
            });
        }
    }

    ScheduleResult {
        events,
        stats,
        finish_time: runfor,
    }
}


fn round_robin(control: [i32; 4], processes: &Vec<Process>) -> ScheduleResult {
    let _process_count = control[0];
    let run_for = control[1];
    let quantum = control[3];

    let mut events = Vec::new();
    let mut stats = Vec::new();
    
    // Tracking structures
    let mut remaining_burst: Vec<i32> = processes.iter().map(|p| p.burst).collect();
    let mut wait_times = vec![0; processes.len()];
    let mut response_times = vec![-1; processes.len()];
    let mut finished_time = vec![-1; processes.len()];
    
    let mut ready_queue: Vec<usize> = Vec::new();
    let mut current_proc_idx: Option<usize> = None;
    let mut current_quantum_left = 0;

    for t in 0..run_for {
        // 1. Handle Arrivals: Add arriving processes to the ready queue
        for i in 0..processes.len() {
            if processes[i].arrival == t {
                events.push(Event {
                    time: t,
                    message: format!("{} arrived", processes[i].name),
                });
                ready_queue.push(i);
            }
        }
        // 2. Manage Execution and Preemption
        if let Some(idx) = current_proc_idx {
            // If process finished in the previous tick
            if remaining_burst[idx] == 0 {
                events.push(Event {
                    time: t,
                    message: format!("{} finished", processes[idx].name),
                });
                finished_time[idx] = t;
                current_proc_idx = None;
            } 
            // If quantum expired, preempt and move to back of queue
            else if current_quantum_left == 0 {
                ready_queue.push(idx);
                current_proc_idx = None;
            }
        }

        // 3. Selection: If CPU is idle, pick the next process from the queue
        if current_proc_idx.is_none() && !ready_queue.is_empty() {
            let next_idx = ready_queue.remove(0);
            current_proc_idx = Some(next_idx);
            current_quantum_left = quantum;

            // Record response time on the first selection
            if response_times[next_idx] == -1 {
                response_times[next_idx] = t - processes[next_idx].arrival;
            }

            events.push(Event {
                time: t,
                message: format!("{} selected (burst {})", processes[next_idx].name, remaining_burst[next_idx]),
            });
        }

        // 4. Tick Logic: Update remaining burst or log Idle
        if let Some(idx) = current_proc_idx {
            remaining_burst[idx] -= 1;
            current_quantum_left -= 1;
        } else {
            events.push(Event {
                time: t,
                message: "Idle".to_string(),
            });
        }

        // Update wait times for all processes sitting in the ready queue
        for &idx in &ready_queue {
            wait_times[idx] += 1;
        }
    }

    // Finalize Metrics and Check for Unfinished Processes
    for i in 0..processes.len() {
        let mut turnaround = 0;
        let mut wait = wait_times[i];

        if finished_time[i] != -1 {
            turnaround = finished_time[i] - processes[i].arrival;
        } else {
            // Requirement: List processes that did not complete within 'runfor'
            events.push(Event {
                time: run_for,
                message: format!("{} did not finish", processes[i].name),
            });
            // Adjust wait time if it never finished? Usually kept as accrued wait.
        }

        stats.push(ProcessStats {
            name: processes[i].name.clone(),
            wait,
            turnaround,
            response: response_times[i],
        });
    }

    ScheduleResult {
        events,
        stats,
        finish_time: run_for,
    }
}


fn linux_cfs(control: [i32; 4], processes: &Vec<Process>) -> ScheduleResult {
    let run_for = control[1];
    let mut events = Vec::new();
    let mut stats = Vec::new();

    // CFS specific data
    let mut vruntime = vec![0.0; processes.len()];
    let mut remaining_burst = processes.iter().map(|p| p.burst).collect::<Vec<_>>();
    let mut finished_time = vec![-1; processes.len()];
    let mut response_times = vec![-1; processes.len()];
    let mut wait_times = vec![0; processes.len()];
    
    let mut ready_indices: Vec<usize> = Vec::new();
    let mut current_proc_idx: Option<usize> = None;

    for t in 0..run_for {
        // 1. Handle Arrivals
        for i in 0..processes.len() {
            if processes[i].arrival == t {
                events.push(Event {
                    time: t,
                    message: format!("{} arrived", processes[i].name),
                });
                ready_indices.push(i);
                
                // New processes start with the current minimum vruntime to be fair
                let min_vruntime = vruntime.iter().cloned().fold(f64::INFINITY, f64::min);
                vruntime[i] = if min_vruntime.is_infinite() { 0.0 } else { min_vruntime };
            }
        }

        // 2. Check if current process is finished
        if let Some(idx) = current_proc_idx {
            if remaining_burst[idx] == 0 {
                events.push(Event {
                    time: t,
                    message: format!("{} finished", processes[idx].name),
                });
                finished_time[idx] = t;
                current_proc_idx = None;
            }
        }

        // 3. Selection: CFS always picks the lowest vruntime
        // In real Linux, this is a Red-Black Tree. Here, we'll search the ready list.
        if !ready_indices.is_empty() {
            // Find process in ready_indices with the smallest vruntime
            let best_ready_pos = ready_indices.iter()
                .enumerate()
                .min_by(|(_, &a), (_, &b)| vruntime[a].partial_cmp(&vruntime[b]).unwrap())
                .map(|(i, _)| i)
                .unwrap();

            let next_idx = ready_indices[best_ready_pos];

            // Preemption logic: If a different process has lower vruntime than current
            if let Some(curr_idx) = current_proc_idx {
                if vruntime[next_idx] < vruntime[curr_idx] {
                    ready_indices.push(curr_idx);
                    current_proc_idx = None; // Trigger re-selection
                }
            }

            if current_proc_idx.is_none() {
                current_proc_idx = Some(ready_indices.remove(best_ready_pos));
                let idx = current_proc_idx.unwrap();
                
                if response_times[idx] == -1 {
                    response_times[idx] = t - processes[idx].arrival;
                }

                events.push(Event {
                    time: t,
                    message: format!("{} selected (burst {}, vruntime {:.2})", 
                                     processes[idx].name, remaining_burst[idx], vruntime[idx]),
                });
            }
        }

        // 4. Tick Logic
        if let Some(idx) = current_proc_idx {
            remaining_burst[idx] -= 1;
            // vruntime += (actual_time / weight). We'll assume weight = 1 for simplicity.
            vruntime[idx] += 1.0; 
        } else {
            events.push(Event { time: t, message: "Idle".to_string() });
        }

        // Update wait times for others
        for &idx in &ready_indices {
            wait_times[idx] += 1;
        }
    }

    // Finalize Stats (same logic as RR)
    for i in 0..processes.len() {
        let mut turnaround = 0;
        if finished_time[i] != -1 {
            turnaround = finished_time[i] - processes[i].arrival;
        } else {
            events.push(Event { time: run_for, message: format!("{} did not finish", processes[i].name) });
        }

        stats.push(ProcessStats {
            name: processes[i].name.clone(),
            wait: wait_times[i],
            turnaround,
            response: response_times[i],
        });
    }

    ScheduleResult { events, stats, finish_time: run_for }
}
