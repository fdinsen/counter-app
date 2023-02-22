use crate::utils::db_handler::{
    add_new_counter, get_all_counters, get_row_id, increment_counter, read_counter,
};

#[derive(PartialEq, Eq)]
pub enum State {
    Exit,
    Cont,
    Load(i32),
}

macro_rules! run_cli {
    ($id: expr) => {{
        let mut state = State::Cont;
        while state == State::Cont {
            let mut line = String::new();
            std::io::stdin().read_line(&mut line).unwrap();
            state = handle_input(&line, $id);
        }
        return state;
    }};
}
pub(crate) use run_cli;

pub fn handle_input(line: &str, current_id: i32) -> State {
    let x = line.split("\r\n").collect::<Vec<&str>>();
    let commands = x[0].split(' ').collect::<Vec<&str>>();
    let cmd = &commands[0].to_lowercase() as &str;
    match cmd {
        "help" => {
            print_help();
            State::Cont
        }
        "list" => {
            list_counters();
            State::Cont
        }
        "add" => {
            let commands = add_counter(commands);
            let next = load_counter(commands);
            State::Load(next)
        }
        "load" => {
            let next = load_counter(commands);
            State::Load(next)
        }
        "" => {
            increment(current_id);
            State::Cont
        }
        "exit" => State::Exit,
        _ => {
            println!("Unknown command {:?}", commands[0]);
            State::Cont
        }
    }
}

fn print_help() {
    println!();
    println!("LIST\t\tLists all the existing counters.");
    println!("ADD name\tAdds a new counter with the given name.");
    println!("LOAD name\tSets the given counter as the active one.");
    println!("EXIT\t\tExits application.");
    println!("Press enter to increment loaded counter.");
    println!("Press Scroll Lock to increment loaded counter when command-line is not in focus.")
}

fn list_counters() {
    let result = get_all_counters();
    let counters = match result {
        Ok(c) => c,
        Err(_) => {
            println!("Error, could not fetch counters.");
            return;
        }
    };
    if !counters.is_empty(){
        for counter in counters {
            println!("{:?}: {:?}", counter.name, counter.counter);
        }
    } else {
        println!("No counters saved. Use ADD command to create new.");
    }
}

fn add_counter(commands: Vec<&str>) -> Vec<&str> {
    if commands.len() > 1 {
        let param = commands[1];
        let res = add_new_counter(param);
        match res {
            Ok(_) => println!("Added {:?} successfully.", param),
            Err(_) => println!("Counter with name {:?} already exists.", param),
        };
    } else {
        println!("Error, no name provided");
    }
    commands
}

fn load_counter(commands: Vec<&str>) -> i32 {
    if commands.len() > 1 {
        let param = commands[1];
        let result = get_row_id(param);
        let id = match result {
            Ok(i) => i,
            Err(_) => {
                println!("Counter with name {:?} does not exist.", param);
                return -1;
            }
        };
        let res = read_counter(id);
        match res {
            Ok(r) => {
                println!("Loaded counter {:?}, current count: {:?}", param, r.counter);
                return id;
            }
            Err(_) => {
                println!("Counter with name {:?} does not exist.", param);
                return -1;
            }
        };
    } else {
        println!("Error, no name provided");
        return -1;
    }
}

pub fn increment(current_id: i32) -> i32 {
    if current_id == -1 {
        println!("No counter loaded");
        return -1;
    }
    let res = increment_counter(current_id);
    match res {
        Ok(count) => {
            println!("{:?}", count);
            return count;
        }
        Err(_) => {
            println!("Error, could not increment.");
            return -1;
        }
    }
}
