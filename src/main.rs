use std::{
    env,
    fs::File,
    io::{Read, Write},
    process::Command,
};

trait RemoveWs {
    fn remove_whitespaces(&self) -> String;
    fn remove_tab(&self) -> String;
}

impl RemoveWs for String {
    fn remove_whitespaces(&self) -> String {
        self.chars().filter(|&c| c != '\n').collect()
    }

    fn remove_tab(&self) -> String {
        let mut res = String::new();
        
        if !self.is_empty() {
            let mut prev: char = self[..1].chars().collect::<Vec<char>>()[0];

            self.chars().for_each(|c| {
                if c == prev {
                    return;
                }

                prev = c;
                res.push(c);
            });
        }

        res
    }
}

fn parse_call(src: &mut String, method: String) {
    let mut bounds: Vec<(usize, usize, String)> = vec![];

    for (i, expr) in src.clone().match_indices(&format!("->{}", method)) {
        let mut offset: usize = expr.len() + 1;

        let var: String = src[..i]
            .chars()
            .rev()
            .take_while(|&c| c != ' ' && c != '\n' && c != ',')
            .collect();

        let mut args: String = src[i..]
            .chars()
            .skip(offset)
            .take_while(|&c| c != ')')
            .collect();

        offset += args.len() + 1;

        if !args.is_empty() {
            args.insert_str(0, ", ");
        }

        let expr = format!(
            "{}({}{})",
            expr,
            var.chars().rev().collect::<String>(),
            args
        );

        bounds.push((i, i + offset, expr));

        println!("call:\n\tmethod: {}\n\targs: {}\n", method, args);
    }

    for (start, end, expr) in bounds.iter().rev() {
        src.replace_range(start..end, expr)
    }
}

fn parse_method(src: &mut String, class: String) {
    let mut bounds: Vec<(usize, usize, String)> = vec![];

    for (i, expr) in src.clone().match_indices(&format!("{}::", class)) {
        let mut offset: usize = expr.len();

        let ty: String = src[..i].chars().rev().take_while(|&c| c != '\n').collect();

        let name: String = src[i..]
            .chars()
            .skip(offset)
            .take_while(|&c| c != '(')
            .collect();

        offset += name.len() + 1;

        let params: String = src[i..]
            .chars()
            .skip(offset)
            .take_while(|&c| c != ')')
            .collect();

        offset += params.len() + 3;

        let body: String = src[i..]
            .chars()
            .skip(offset)
            .take_while(|&c| c != '}')
            .collect();

        offset += body.len() + 1;

        let expr = format!(
            include_str!("tmpl/method.tmpl"),
            class.trim(),
            ty.chars().rev().collect::<String>().trim(),
            name.trim(),
            params.replace(",", ";"),
            body.trim()
        );

        bounds.push((i - ty.len(), i + offset, expr));
        parse_call(src, name.clone());

        println!(
            "method:\n\tty: {}\n\tname: {}\n\tparams: {}\n\tbody: {}\n",
            ty.chars().rev().collect::<String>(),
            name,
            params.remove_whitespaces().remove_tab(),
            body.remove_whitespaces().remove_tab()
        );
    }

    for (start, end, expr) in bounds.iter().rev() {
        src.replace_range(start..end, expr);
    }
}

fn parse_constructor(src: &mut String) {
    let mut bounds: Vec<(usize, usize, String)> = vec![];

    for (i, op) in src.clone().match_indices("new") {
        let mut offset: usize = op.len() + 1;

        let class: String = src[i..]
            .chars()
            .skip(offset)
            .take_while(|&c| c != '(')
            .collect();

        offset += class.len() + 1;

        let body: String = src[i..]
            .chars()
            .skip(offset)
            .take_while(|&c| c != ')')
            .collect();

        offset += body.len() + 2;

        let expr = format!(include_str!("tmpl/new.tmpl"), class, body);

        bounds.push((i, i + offset, expr));

        println!("intialization:\n\tclass: {}\n\targs: {}", class, body);
    }

    for (start, end, expr) in bounds.iter().rev() {
        src.replace_range(start..end, expr);
    }
}

fn parse_classes(src: &mut String) {
    let mut bounds: Vec<(usize, usize, String)> = vec![];

    for (i, op) in src.clone().match_indices("class") {
        let mut offset: usize = op.len() + 1;

        let name: String = src[i..]
            .chars()
            .skip(offset)
            .take_while(|&c| c != ' ' && c != '{')
            .collect();

        offset += name.len() + 1;

        let parent: String = src[i..]
            .chars()
            .skip(offset)
            .take_while(|&c| c != '{')
            .collect();

        offset += parent.len() + 1;

        let mut body: String = src[i..]
            .chars()
            .skip(offset)
            .take_while(|&c| c != '}')
            .collect();

        offset += body.len() + 1;

        if parent.trim() != "" {
            body.push_str(&format!("\n  {} super;\n", parent.trim_matches('+')));
        }

        let expr = format!(include_str!("tmpl/class.tmpl"), name, &body[1..]);

        bounds.push((i, i + offset, expr));

        parse_method(src, name.clone().trim().to_string());

        println!(
            "class declaration\n\tclass: {}\n\tfields: {}\n",
            name.trim(),
            body.remove_tab().remove_whitespaces()
        );
    }

    for (start, end, expr) in bounds.iter().rev() {
        src.replace_range(start..end, expr);
    }

    src.insert_str(0, "#include \"src/c/include/class.h\"\n");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("USAGE {} FILE", args[0]);
        return;
    }

    if !args[1].ends_with(".preC") {
        eprintln!("FILE must be PreC source file");
        return;
    }

    let mut buffer = String::new();

    match File::open(args[1].clone()).and_then(|mut file| file.read_to_string(&mut buffer)) {
        Ok(_) => {
            parse_classes(&mut buffer);
            parse_constructor(&mut buffer);

            // buffer = buffer.remove_end_line();

            match File::create("pre.c").and_then(|mut file| file.write(buffer.as_bytes())) {
                Ok(_) => {
                    Command::new("clang")
                        .args(["-o", "program.exe", "pre.c"])
                        .output()
                        .expect("msg");
                    buffer.clear();
                }
                Err(_) => eprintln!("Failed to run compiled program"),
            }
        }
        Err(_) => eprintln!("Unable to read source file"),
    }
}
