fn sum_with_step(total : &mut i32, low: i32, high: i32, step: i32) {
    for n in 0..=((high - low) / step) {
        *total += low + step * n;
    }
}

fn assignment_1_main() {
    let mut result = 0;
    sum_with_step(&mut result, 0, 100, 1);
    println!("Sum 0 to 100, step 1: {}", result);

    result = 0;
    sum_with_step(&mut result, 0, 10, 2);
    println!("Sum 0 to 10, step 2: {}", result);

    result = 0;
    sum_with_step(&mut result, 5, 15, 3);
    println!("Sum 5 to 15, step 3: {}", result);
}

fn most_frequent_word(text: &str) -> (String, usize) {
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut frequencies: Vec<(String, usize)> = Vec::new();
    let mut most_frequent_word: String = String::new();
    let mut max_frequency: usize = 0;
    for word in words {
        let mut recorded: bool = false;
        for (entry, frequency) in &mut frequencies {
            if word == entry {
                recorded = true;
                *frequency += 1;
                if *frequency > max_frequency {
                    max_frequency = *frequency;
                    most_frequent_word = word.to_string();
                }
                break;
            }
        }
        if !recorded {
            frequencies.push((word.to_string(), 1));
        }
    }
    (most_frequent_word, max_frequency)
}

fn assignment_2_main() {
    let text = "the quick brown fox jumps over the lazy dog the quick brown fox";
    let (word, count) = most_frequent_word(text);
    println!("Most frequent word: \"{}\" ({} times)", word, count);
}

fn main() {
    //assignment_1_main();
    assignment_2_main();
}
