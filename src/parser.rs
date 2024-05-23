#[derive(Debug)]
struct Class {
    name: String,
    methods: Vec<String>,
}

pub struct Parser {
    pub src: String,
    classes: Vec<Class>,
}

impl Parser {
    pub fn new(src: String) -> Self {
        Self {
            src,
            classes: vec![],
        }
    }

    fn collect_until<F>(&self, start: usize, n: usize, condition: F) -> String
    where
        F: Fn(char) -> bool,
    {
        self.src[start..]
            .chars()
            .skip(n)
            .take_while(|&c| !condition(c))
            .collect()
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
            let mut offset: usize = expr.len() + 1;

            let var: String = self.src[..i]
                .chars()
                .rev()
                .take_while(|&c| c != ' ' && c != '\n' && c != ',')
                .collect();

            let mut args: String = self.collect_until(i, offset, |c| c == ')');

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

            //dbg!(method.clone(), args);
        }

        self.insert(bounds);
    }

    fn parse_method(&mut self, class: String) {
        let mut bounds: Vec<(usize, usize, String)> = vec![];

        for (i, expr) in self.src.clone().match_indices(&format!("{}::", class)) {
            let mut offset: usize = expr.len();

            let ty: String = self.src[..i]
                .chars()
                .rev()
                .take_while(|&c| c != '\n')
                .collect();

            let name: String = self.collect_until(i, offset, |c| c == '(');

            offset += name.len() + 1;

            let params: String = self.collect_until(i, offset, |c| c == ')');

            offset += params.len() + 3;

            let mut body: String = self.collect_until(i, offset, |c| c == '}');

            offset += body.len() + 1;

            body = body.replace("super", "self->super");

            let expr = format!(
                include_str!("tmpl/method.tmpl"),
                class.trim(),
                ty.chars().rev().collect::<String>().trim(),
                name.trim(),
                params.replace(",", ";"),
                body
            );

            bounds.push((i - ty.len(), i + offset, expr));

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
            let mut offset: usize = op.len() + 1;

            let class: String = self.collect_until(i, offset, |c| c == '(');

            offset += class.len() + 1;

            let mut body: String = self.collect_until(i, offset, |c| c == ')');

            offset += body.len() + 2;

            if let Some(class) = self.classes.iter().find(|c| c.name == class) {
                let body_start: String = body.chars().take_while(|&c| c != '{').collect();
                let body_end: String = body.chars().rev().take_while(|&c| c != ',').collect();

                let mut methods = String::new();

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

            bounds.push((i, i + offset, expr));

            //dbg!(class, body);
            dbg!(&self.classes);
        }

        self.insert(bounds);
    }

    fn parse_classes(&mut self) {
        let mut bounds: Vec<(usize, usize, String)> = vec![];

        for (i, op) in self.src.clone().match_indices("class") {
            let mut offset: usize = op.len() + 1;

            let name: String = self.collect_until(i, offset, |c| c == ' ' || c == '{');

            offset += name.len() + 1;

            let parent: String = self.collect_until(i, offset, |c| c == '{');

            offset += parent.len() + 1;

            let mut body: String = self.collect_until(i, offset, |c| c == '}');

            offset += body.len() + 1;

            if !parent.trim().is_empty() {
                body.push_str(&format!("\n\t{} super;\n", parent.trim_matches('+').trim()));
            }

            let expr = format!(include_str!("tmpl/class.tmpl"), name, &body[1..]);

            bounds.push((i, i + offset, expr));

            self.classes.push(Class {
                name: name.clone(),
                methods: vec![],
            });
            self.parse_method(name.clone().trim().to_string());

            //dbg!(name, body);
        }

        self.insert(bounds);

        // self.src
        //     .insert_str(0, "#include \"src/c/include/class.h\"\n");
        self.src.insert_str(0, include_str!("c/include/class.h"))
    }
}
