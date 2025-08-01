fn fahrenheit_to_celsius(f: f64) -> f64 {
    (f - 32.0) / 1.8
}

fn celsius_to_fahrenheit(c: f64) -> f64 {
    c * 1.8 + 32.0
}

fn assignment_1_main() {
    const STARTING_TEMPERATURE: f64 = -40.0;
    let mut temperature: f64 = STARTING_TEMPERATURE;
    temperature = fahrenheit_to_celsius(temperature);
    println!("{} °F is equal to {} °C",
             STARTING_TEMPERATURE,
             temperature);
    for increment in 1..=5 {
        temperature = STARTING_TEMPERATURE + increment as f64;
        println!("{} °F is equal to {} °C",
                 temperature,
                 fahrenheit_to_celsius(temperature));
    }
}

fn is_even(n: i32) -> bool { n % 2 == 0 }

fn assignment_2_main() {
    let nums: [i32; 10] = [0,1,2,-1,120,3,4,16,240,20];
    let fizzbuzz_map: [(i32, &str); 2] = [(3,"Fizz"),(5,"Buzz")];
    let mut display: String = String::new();
    let mut modified: bool = false;
    for n in nums {
        for (key, value) in fizzbuzz_map {
            if n % key == 0 { 
                display += value;
                modified = true;
            }
        }
        if modified {
            println!("{}", display);
        } else {
            println!("{} is {}", n, if is_even(n) {"even"} else {"odd"});
        }
        display.clear();
        modified = false;
    }
}

fn check_guess(guess: i32, secret: i32) -> i32 {
    match guess - secret {
        ..0 => -1,
        0   =>  0,
        1.. =>  1
    }
}

fn assignment_3_main() {
    let mut secret: i32 = -2;
    let mut guess: i32;
    let mut iterations: usize = 1;
    let mut check_result: i32;
    let simulated_input: [i32; 11] = [0,1,2,-1,120,3,4,-16,-240,20,-2];
    loop {
        guess = simulated_input[iterations-1];
        check_result = check_guess(guess, secret);
        if check_result == 0 {
            println!("Correct!");
            break;
        } else if check_result < 0 {
            println!("Too low");
        } else {
            println!("Too high");
        }
        iterations += 1;
    }
    println!("Took {} guesses.", iterations);
}

fn main() {
    //assignment_1_main();
    //assignment_2_main();
    assignment_3_main();
}
