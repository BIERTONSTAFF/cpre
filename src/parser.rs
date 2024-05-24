#[derive(Debug)]
struct Class {
    name: String,
    methods: Vec<String>,
}

pub struct Parser {
    pub src: String,
    offset: usize,
    classes: Vec<Class>,
}

impl Parser {
    pub fn new(src: String) -> Self {
        Self {
            src,
            offset: 0,
            classes: vec![],
        }
    }

    fn collect_until<F>(&mut self, start: usize, a: usize, condition: F) -> String
    where
        F: Fn(char) -> bool,
    {
        let res: String = self.src[start..]
            .chars()
            .skip(self.offset)
            .take_while(|&c| !condition(c))
            .collect();

        self.offset += res.len() + a;

        res
    }

    fn insert(&mut self, bounds: Vec<(usize, usize, String)>) {
        for (start, end, expr) in bounds.iter().rev() {
            self.src.replace_range(*start..*end, expr);
        }
    }

    pub fn parse(&mut self) {
        self.parse_classes();
        self.parse_constructor();
    }

    fn parse_call(&mut self, method: String) {
        let mut bounds: Vec<(usize, usize, String)> = vec![];

        for (i, expr) in self.src.clone().match_indices(&format!("->{}", method)) {
            self.offset = expr.len() + 1;

            let var: String = self.src[..i]
                .chars()
                .rev()
                .take_while(|&c| c != ' ' && c != '\n' && c != ',')
                .collect();

            let mut args: String = self.collect_until(i, 1, |c| c == ')');

            if !args.is_empty() {
                args.insert_str(0, ", ");
            }

            let expr = format!(
                "{}({}{})",
                expr,
                var.chars().rev().collect::<String>(),
                args
            );

            bounds.push((i, i + self.offset, expr));

            //dbg!(method.clone(), args);
        }

        self.insert(bounds);
    }

    fn parse_method(&mut self, class: String) {
        let mut bounds: Vec<(usize, usize, String)> = vec![];

        for (i, expr) in self.src.clone().match_indices(&format!("{}::", class)) {
            self.offset = expr.len();

            let ty: String = self.src[..i]
                .chars()
                .rev()
                .take_while(|&c| c != '\n')
                .collect();
            let name: String = self.collect_until(i, 1, |c| c == '(');
            let params: String = self.collect_until(i, 3, |c| c == ')');
            let mut body: String = self.collect_until(i, 1, |c| c == '}');

            body = body.replace("super", "self->super");

            let expr = format!(
                include_str!("tmpl/method.tmpl"),
                class.trim(),
                ty.chars().rev().collect::<String>().trim(),
                name.trim(),
                params.replace(",", ";"),
                body
            );

            bounds.push((i - ty.len(), i + self.offset, expr));

            if let Some(class) = self.classes.iter_mut().find(|c| c.name == class) {
                if !class.methods.contains(&name) {
                    class.methods.push(name.clone());
                }
            } else {
                self.classes.push(Class {
                    name: class.clone(),
                    methods: vec![name.clone()],
                });
            }


            self.parse_call(name.clone());

            //dbg!(ty.chars().rev().collect::<String>(), name, params, body);
        }

        self.insert(bounds);
    }

    fn parse_constructor(&mut self) {
        let mut bounds: Vec<(usize, usize, String)> = vec![];

        for (i, op) in self.src.clone().match_indices("new") {
            self.offset = op.len() + 1;

            let class: String = self.collect_until(i, 1, |c| c == '(');
            let mut body: String = self.collect_until(i, 2, |c| c == ')');

            if let Some(class) = self.classes.iter().find(|c| c.name == class) {
                let body_start: String = body.chars().take_while(|&c| c != '{').collect();
                let body_end: String = match body.contains("{") {
                    true => body.chars().rev().take_while(|&c| c != ',').collect(),
                    false => String::new(),
                };

                let mut methods = match body.contains(',') {
                    true => String::new(),
                    false => ", ".to_string(),
                };

                class.methods.iter().enumerate().for_each(|(i, m)| {
                    methods.push_str(&format!("{}_{},", class.name, m));

                    if i != class.methods.len() - 1 {
                        methods.push_str(" ");
                    }
                });

                body = format!(
                    "{}{}{}",
                    body_start,
                    methods,
                    body_end.chars().rev().collect::<String>()
                );
            }

            let expr = format!(include_str!("tmpl/new.tmpl"), class, body);

            bounds.push((i, i + self.offset, expr));

            //dbg!(class, body);
            dbg!(&self.classes);
        }

        self.insert(bounds);
    }

    fn parse_classes(&mut self) {
        let mut bounds: Vec<(usize, usize, String)> = vec![];

        for (i, op) in self.src.clone().match_indices("class") {
            self.offset = op.len() + 1;

            let name: String = self.collect_until(i, 1, |c| c == ' ' || c == '{');
            let parent: String = self.collect_until(i, 1, |c| c == '{');
            let mut body: String = self.collect_until(i, 1, |c| c == '}');

            if !parent.trim().is_empty() {
                body.push_str(&format!("\n\t{} super;\n", parent.trim_matches('+').trim()));
            }

            let expr = format!(include_str!("tmpl/class.tmpl"), name, &body[1..]);

            bounds.push((i, i + self.offset, expr));

            self.classes.push(Class {
                name: name.clone(),
                methods: vec![],
            });
            self.parse_method(name.clone().trim().to_string());

            //dbg!(name, body);
        }

        self.insert(bounds);
        self.src.insert_str(0, include_str!("c/include/class.h"))
    }
}
