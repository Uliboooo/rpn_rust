fn main() {
    loop {
        println!("Please input formula.\nex: 1 + 2 is 1 2 +\nDelimiter is half-width space.");
        let input_formula = get_input();
        let delimited_input_fomula = delimit(&input_formula);
        let mut stack = Vec::<f64>::new();
        let mut result = 0.0;
        let mut double_operator_count = false;

        for i in delimited_input_fomula {
            if is_numeric(i) == true {
                double_operator_count = false;
                stack.push(i.parse::<f64>().unwrap_or(0.0));
            } else { //演算子の場合
                if double_operator_count == false {
                    // let last_index = stack.len() - 1;
                    result =  calculation(stack[stack.len() - 2], stack[stack.len() - 1], i);
                    for _ in 0..2 {
                        stack.remove(stack.len() - 1);
                    }
                    stack.push(result); //結果の挿入
                }
            }
        }
        println!("{}\nAgain?(y/n)", result);
        if get_input() == "n".to_string() {
            break;
        }
    }
}

fn get_input() -> String { //String型で入力を返す
    let mut word = String::new();
    std::io::stdin()
        .read_line(&mut word)
        .expect("Failed to read line");
    word.trim().to_string()
}

fn delimit(input: &String) -> Vec<&str>{ //文字列を空白で区切りベクタにして返す
    input.split_whitespace().collect()
}

fn is_numeric(input: &str) -> bool { //入力が数値ならtrue, 演算子ならfalse
    match input.parse::<f64>() {
        Ok(_) => true,
        Err(_) => false,
    }
}

fn calculation(operand_1: f64, operand_2: f64, operator: &str) -> f64 { //演算
    match operator {
        "+" => operand_1 + operand_2,
        "-" => operand_1 - operand_2,
        "*" => operand_1 * operand_2,
        "/" => operand_1 / operand_2,
        "%" => operand_1 % operand_2,
        "**" => power(operand_1, operand_2),
        _ => 0.0,
    }
}

fn power(operand_1: f64, operand_2: f64) -> f64 {
    let mut power_result = operand_1;
    for _ in 0..operand_2 as i64 - 1 {
        power_result *= operand_1
    }
    power_result
}