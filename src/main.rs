fn main() {
    loop {
        println!("Please input formula.\nex: 1 + 2 is 1 2 +\nDelimiter is half-width space.");
        let input_formula = get_input();
        let delimited_input_fomula = delimit(&input_formula);
        // let mut operands = Vec::<f64>::new();
        let mut stack = Vec::<f64>::new();
        let mut result = 0.0;
        let mut double_operator_count = false;
        // let mut index = 0;

        for i in delimited_input_fomula {
            if is_numeric(i) == true {
                double_operator_count = false;
                stack.push(i.parse::<f64>().unwrap_or(0.0));
            } else { //演算子の場合
                if double_operator_count == false {
                    let last_index = stack.len() - 1;
                    println!("ope1: {}\nope2: {}", last_index - 1, last_index);
                    result =  calculation(stack[last_index - 1], stack[last_index], i);
                    println!("stack: {:?}", stack); //デバッグ用
                    println!("ope1: {}\nope2: {}", last_index - 1, last_index);
                    stack.remove(last_index - 1); //ope1の削除
                    stack.remove(last_index); //ope2の削除
                    stack.push(result); //結果の挿入
                    // stack.remove(index + 1);
                    // operands.clear();
                    // stack.push(result);
                }
                // println!("result stack: {:?}", stack); //デバッグ用
                // if double_operator_count == true {
                //     result = calculation(stack[index], stack[index + 1], i);
                //     stack.clear();
                //     stack.push(result);
                //     // result_stack.clear();
                //     // index += 1;
                // }
                // double_operator_count = true;
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

// fn check_syntax(checked_input: String) {
//     for c in delimit(checked_input) { //順番にできるようにする。添え字?
//         if is_numeric(&c.to_sring()) == true { //数値の場合
//         } else { //演算子の場合
//         }
//     }
// }

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