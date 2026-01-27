use intentscript_parser::*;

/// Format an IntentScript AST back to source code
pub struct Formatter {
    indent_level: usize,
    indent_size: usize,
}

impl Formatter {
    pub fn new() -> Self {
        Self {
            indent_level: 0,
            indent_size: 2,
        }
    }

    fn indent(&self) -> String {
        " ".repeat(self.indent_level * self.indent_size)
    }

    fn increase_indent(&mut self) {
        self.indent_level += 1;
    }

    fn decrease_indent(&mut self) {
        if self.indent_level > 0 {
            self.indent_level -= 1;
        }
    }

    /// Format a complete file
    pub fn format_file(&mut self, file: &File) -> String {
        let mut output = String::new();

        for (i, task) in file.tasks.iter().enumerate() {
            if i > 0 {
                output.push_str("\n\n");
            }
            output.push_str(&self.format_task(task));
        }

        output
    }

    /// Format a task
    fn format_task(&mut self, task: &Task) -> String {
        let mut output = String::new();

        output.push_str("task ");
        output.push('"');
        output.push_str(&task.name);
        output.push('"');

        if let Some(version) = &task.version {
            output.push(' ');
            output.push('v');
            output.push_str(&version.major.to_string());
            output.push('.');
            output.push_str(&version.minor.to_string());
            if let Some(patch) = version.patch {
                output.push('.');
                output.push_str(&patch.to_string());
            }
        }

        output.push_str(" {\n");
        self.increase_indent();

        for section in &task.sections {
            output.push_str(&self.format_section(section));
            output.push('\n');
        }

        self.decrease_indent();
        output.push('}');

        output
    }

    /// Format a section
    fn format_section(&mut self, section: &Section) -> String {
        match section {
            Section::Goal(expr) => {
                format!("{}goal: {}", self.indent(), self.format_expr(expr))
            }
            Section::Input(inputs) => self.format_input_section(inputs),
            Section::Constraints(constraints) => self.format_constraints_section(constraints),
            Section::OutputSchema(type_expr) => {
                format!(
                    "{}output_schema: {}",
                    self.indent(),
                    self.format_type_expr(type_expr)
                )
            }
            Section::Checks(checks) => self.format_checks_section(checks),
            Section::Run(pipeline) => {
                format!("{}run: {}", self.indent(), self.format_pipeline(pipeline))
            }
        }
    }

    /// Format input section
    fn format_input_section(&mut self, inputs: &[InputDecl]) -> String {
        let mut output = String::new();
        output.push_str(&self.indent());
        output.push_str("input: ");

        if inputs.len() == 1 {
            // Inline format for single input
            output.push_str(&self.format_input_decl(&inputs[0]));
        } else {
            // Block format for multiple inputs
            output.push_str("{\n");
            self.increase_indent();
            for (i, input) in inputs.iter().enumerate() {
                output.push_str(&self.indent());
                output.push_str(&self.format_input_decl(input));
                if i < inputs.len() - 1 {
                    output.push(',');
                }
                output.push('\n');
            }
            self.decrease_indent();
            output.push_str(&self.indent());
            output.push('}');
        }

        output
    }

    /// Format a single input declaration
    fn format_input_decl(&self, input: &InputDecl) -> String {
        let mut output = String::new();
        output.push_str(&input.name);
        output.push_str(": ");
        output.push_str(&self.format_type_expr(&input.type_expr));

        if let Some(default) = &input.default {
            output.push_str(" = ");
            output.push_str(&self.format_literal(default));
        }

        output
    }

    /// Format constraints section
    fn format_constraints_section(&mut self, constraints: &[ConstraintDecl]) -> String {
        let mut output = String::new();
        output.push_str(&self.indent());
        output.push_str("constraints: {\n");
        self.increase_indent();

        for (i, constraint) in constraints.iter().enumerate() {
            output.push_str(&self.indent());
            output.push_str(&constraint.name);
            output.push_str(" = ");
            output.push_str(&self.format_constraint_value(&constraint.value));
            if i < constraints.len() - 1 {
                output.push(',');
            }
            output.push('\n');
        }

        self.decrease_indent();
        output.push_str(&self.indent());
        output.push('}');

        output
    }

    /// Format a constraint value
    fn format_constraint_value(&self, value: &ConstraintValue) -> String {
        match value {
            ConstraintValue::On => "on".to_string(),
            ConstraintValue::Off => "off".to_string(),
            ConstraintValue::Literal(lit) => self.format_literal(lit),
            ConstraintValue::Expr(expr) => self.format_expr(expr),
        }
    }

    /// Format checks section
    fn format_checks_section(&mut self, checks: &[CheckDecl]) -> String {
        let mut output = String::new();
        output.push_str(&self.indent());
        output.push_str("checks: {\n");
        self.increase_indent();

        for (i, check) in checks.iter().enumerate() {
            output.push_str(&self.indent());
            output.push_str(&check.name);
            output.push('(');
            for (j, arg) in check.args.iter().enumerate() {
                output.push_str(&self.format_expr(arg));
                if j < check.args.len() - 1 {
                    output.push_str(", ");
                }
            }
            output.push(')');
            if i < checks.len() - 1 {
                output.push(',');
            }
            output.push('\n');
        }

        self.decrease_indent();
        output.push_str(&self.indent());
        output.push('}');

        output
    }

    /// Format an expression
    fn format_expr(&self, expr: &Expr) -> String {
        match expr {
            Expr::Literal(lit, _) => self.format_literal(lit),
            Expr::Ident(name, _) => name.clone(),
            Expr::Call(call) => self.format_call_expr(call),
        }
    }

    /// Format a literal
    fn format_literal(&self, literal: &Literal) -> String {
        match literal {
            Literal::String(s) => format!("\"{}\"", escape_string(s)),
            Literal::Int(i) => i.to_string(),
            Literal::Float(f) => f.to_string(),
            Literal::Bool(b) => b.to_string(),
        }
    }

    /// Format a call expression
    fn format_call_expr(&self, call: &CallExpr) -> String {
        let mut output = String::new();
        output.push_str(&call.name);
        output.push('(');

        for (i, arg) in call.args.iter().enumerate() {
            match arg {
                Arg::Named { name, value } => {
                    output.push_str(name);
                    output.push_str(" = ");
                    output.push_str(&self.format_expr(value));
                }
                Arg::Positional(expr) => {
                    output.push_str(&self.format_expr(expr));
                }
            }
            if i < call.args.len() - 1 {
                output.push_str(", ");
            }
        }

        output.push(')');
        output
    }

    /// Format a type expression
    fn format_type_expr(&self, type_expr: &TypeExpr) -> String {
        match type_expr {
            TypeExpr::Primitive(prim, _) => format!("{:?}", prim).to_lowercase(),
            TypeExpr::Domain(domain, _) => format!("{:?}", domain).to_lowercase(),
            TypeExpr::List(inner, _) => format!("list[{}]", self.format_type_expr(inner)),
            TypeExpr::Optional(inner, _) => format!("optional[{}]", self.format_type_expr(inner)),
            TypeExpr::Enum(variants, _) => {
                let variant_strs: Vec<_> = variants.iter().map(|v| format!("\"{}\"", v)).collect();
                format!("enum[{}]", variant_strs.join(", "))
            }
            TypeExpr::Object { fields, .. } => {
                let field_strs: Vec<_> = fields
                    .iter()
                    .map(|(name, ty)| format!("{}: {}", name, self.format_type_expr(ty)))
                    .collect();
                format!("object {{ {} }}", field_strs.join(", "))
            }
        }
    }

    /// Format a pipeline
    fn format_pipeline(&self, pipeline: &Pipeline) -> String {
        let step_strs: Vec<_> = pipeline.steps.iter().map(|s| self.format_step(s)).collect();
        step_strs.join(" -> ")
    }

    /// Format a pipeline step
    fn format_step(&self, step: &Step) -> String {
        match step {
            Step::Call(call) => self.format_call_expr(call),
            Step::Ident(name, _) => name.clone(),
        }
    }
}

impl Default for Formatter {
    fn default() -> Self {
        Self::new()
    }
}

/// Escape special characters in a string
fn escape_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\t', "\\t")
}
