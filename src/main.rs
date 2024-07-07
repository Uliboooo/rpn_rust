use regex::Regex;
use get_input::get_input;
use colored::Colorize;
use std::fs::OpenOptions;
use std::io::BufRead;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Write;
use std::path::Path;
use chrono::Local;

struct History { //日付、成否、入力された式、結果もしくはエラーコード
    date: String,
    is_success: bool,
    forumula: String,
    solution: f64,
}

struct SolutionResult {
    solution: f64,
    success: bool,
}

fn show_error(error_code_num: u16) { //エラーコードから適切なエラーを表示
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
    Ok(
        match operator {
        "+" => operand_1 + operand_2,
        "-" => operand_1 - operand_2,
        "*" => operand_1 * operand_2,
        "/" => operand_1 / operand_2,
        "%" => operand_1 % operand_2,
        "**" => power(operand_1, operand_2),
        _ => return Err(0201),
        }
    )
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

fn bool_to_string(is_success: bool) -> String { //trueなら"成功"、falseなら"失敗"を返す
    if is_success == true {"成功".to_string()} else {"失敗".to_string()}
}

fn history_to_string(content: History) -> String { //書き込み可能なStringに変換
    let mut log_content = content.date.to_string();
    log_content.push_str(bool_to_string(content.is_success).as_str());
    log_content.push_str(",");
    log_content.push_str(content.forumula.as_str());
    log_content.push_str(",");
    log_content.push_str(content.solution.to_string().as_str());
    log_content.push_str("\n");
    log_content
}

fn add_data_csv(path: &Path, content: History) {
    let file = match OpenOptions::new().write(true).append(true).open(path){
        Ok(file) => file,
        Err(_) => return,
    };
    let mut bw = BufWriter::new(file);
    match bw.write(history_to_string(content).as_bytes()) {
        Ok(_) => {
            match bw.flush(){
                Ok(_) => {},
                Err(_) => return,
            } },
        Err(_) => return,
    }
}

fn add_column_csv(path: &Path) -> Result<(), std::io::Error> {
    let file = OpenOptions::new().create(true).read(true).write(true).open(path)?;
    let reader = BufReader::new(file);
    let column = "日付,成否,入力された式,結果(成否が失敗の場合はエラーコード)";
    let mut lines: Vec<String> = reader.lines().collect::<Result<_, _>>()?;
    if lines.is_empty() || lines[0] != column {
        lines.insert(0, column.to_string());
        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(path)?;
        for line in lines {
            writeln!(file, "{}", line)?;
        }
    }
    Ok(())
}

fn log_history(log_content: History) {
    let path = Path::new("history.csv");
    match add_column_csv(path) {
        Ok(_) => {()},
        Err(_) => return,
    };
    add_data_csv(path, log_content);
}
fn main() {
    loop {
        println!("式を入力してください。\"n\"で終了\n例: 1 2 + (値や演算子は半角スペースで区切ってください。)\n使用可能演算子: 加(+)減(-)乗(*)除(/)余(%)指(**)");
        let input_formula = get_input();
        if &input_formula == &"n".to_string() {break;};
        let result = match check_syntax(&input_formula) {
            Ok(_) => {
                let delimited_input_fomula = delimit(&input_formula);
                match stack_manage(delimited_input_fomula) {
                    Ok(result) => {
                        SolutionResult {
                            solution: result,
                            success: true
                        }
                    },
                    Err(error_code) => {
                        show_error(error_code);
                        SolutionResult {
                            solution: error_code as f64,
                            success: false,
                        }
                    },
                }
            },
            Err(error_code) => {
                show_error(error_code);
                SolutionResult {
                    solution: error_code as f64,
                    success: false,
                }
            },
        };
        if result.success == true {
            println!("{}\n", result.solution);
        }
        log_history(
            History {
                date: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                is_success: result.success,
                forumula: input_formula.clone(),
                solution: result.solution
            }
        );
    }
}