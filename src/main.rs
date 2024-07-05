use regex::Regex;
use get_input::get_input;
use colored::Colorize;

fn show_error(error_code_num: u16) {
    eprintln!(
        "{}{} {}",
        error_code_num.to_string().red(),
        ":".red(),
        match error_code_num {
            0101 => "計算不可能な文字が含まれています。",
            0102 => "式が入力されていない可能性があります。",
            0103 => "演算子の間にスペースが含まれていない可能性があります。",
            0104 => "被演算子(数が足りない可能性があります。",
            0105 => "数値に変換できませんでした。",
            0106 => "演算子が入力されていない可能性があります。",
            0201 => "未定義の演算子が入力されました。",
            _ => "原因不明のエラーです",
        }.to_string().red(),
    );
    println!("もう一度入力してください。\n");
}

fn check_unavailable_character(checked_string: &String) -> bool { //入力に演算不可能な文字があった場合false
    let re = Regex::new("[^+\\-*/%1234567890 ]").unwrap();
    if re.is_match(&checked_string) {
        false
    } else {
        true
    }
}

fn check_is_operator (checked_string: &String) -> bool { //演算子が存在しない場合
    let re = Regex::new(r"[+\-*/%]").unwrap();
    if re.is_match(&checked_string){
        true
    } else {
        false
    }
}

fn check_length(checked_string: &String) -> bool{ //式が入力されていない場合
    if checked_string.len() <= 1 {
        false
    } else {
        true
    }
}

fn check_halfspace(checked_string: &String) -> bool { //演算子の間のスペース
    let re = Regex::new(r"\d[^\w\s]").unwrap();
    if re.is_match(&checked_string) {
        false
    } else {
        true
    }
}

fn check_syntax(checked_string: &String) -> bool { //入力された式のチェック
    if check_unavailable_character(checked_string) == false {
        show_error(0101);
        false
    } else if check_length(checked_string) == false {
        show_error(0102);
        false
    } else if check_halfspace(checked_string) == false {
        show_error(0103);
        false
    } else if check_is_operator(checked_string) == false {
        show_error(0106);
        false
    } else {
        true
    }
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

fn to_num(input_str: &str) -> Result<f64, u16> {
    match input_str.parse::<f64>() {
        Ok(n) => Ok(n),
        Err(_) => Err(0105),
    }
}

fn calculation(operand_1: f64, operand_2: f64, operator: &str) -> Result<f64, u16> { //演算
    match operator {
        "+" => Ok(operand_1 + operand_2),
        "-" => Ok(operand_1 - operand_2),
        "*" => Ok(operand_1 * operand_2),
        "/" => Ok(operand_1 / operand_2),
        "%" => Ok(operand_1 % operand_2),
        "**" => Ok(power(operand_1, operand_2)),
        _ => Err(0201),
    }
}

fn power(operand_1: f64, operand_2: f64) -> f64 { //指数演算
    let mut power_result = operand_1;
    for _ in 0..operand_2 as i64 - 1 {
        power_result *= operand_1
    }
    power_result
}

fn stack_manage(delimited_input: Vec<&str>) -> Result<f64, u16>{
    let mut stack = Vec::<f64>::new();
    for i in delimited_input {
        if is_numeric(i) == true { //オペランドの場合
            stack.push(
                match to_num(i) {
                    Ok(result) => result,
                    Err(error_code) => return Err(error_code),
                }
            );
        } else { //演算子の場合
            if stack.len() < 2 {return Err(0104)}; //引数不足
            let result = match calculation(stack[stack.len() - 2], stack[stack.len() - 1], i) {
                Ok(result_) => result_,
                Err(error_code) => return Err(error_code),
            };
            for _ in 0..2 {
                stack.remove(stack.len() - 1); //stackのクリア
            }
            stack.push(result); //結果の挿入
        }
    }
    if stack.len() > 1 {
        Err(0104)
    } else {
        Ok(stack[stack.len() - 1])
    }
}

fn main() {
    loop {
        println!("式を入力してください。\"n\"で終了\n例: 1 2 + (値や演算子は半角スペースで区切ってください。)\n使用可能演算子: 加(+)減(-)乗(*)除(/)余(%)指(**)");
        let input_formula = get_input();
        if &input_formula == &"n".to_string() {break;};
        if check_syntax(&input_formula) == false {continue;};
        let delimited_input_fomula = delimit(&input_formula);
        let result = match stack_manage(delimited_input_fomula) {
            Ok(result_) => result_,
            Err(error_code) => {
                show_error(error_code);
                continue;
            }
        };
        println!("{}\nもう一度計算しますか?(y/n)", result);
        if get_input() == "n".to_string() {
            break;
        }
        println!("");
    }
}