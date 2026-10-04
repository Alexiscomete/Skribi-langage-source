use log::error;
use std::fmt::{Display, Error, Formatter};

use crate::{
    ast::{
        nodes::FileTreeRoot,
        visitors::{AstMutVisitor, expression_max_depth::expression_max_depth},
    },
    interner::get_interner_typed,
};

struct PrettyPrinterVisitor<'fmt_ref, 'fmt_object> {
    f: &'fmt_ref mut Formatter<'fmt_object>,
    indent: usize,
}

impl Display for FileTreeRoot {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut printer = PrettyPrinterVisitor { f, indent: 0 };
        if self.file.is_none() {
            error!("No file detected while formatting the AST");
        }
        printer.visit_file_tree_root(self)
    }
}

const IDENT: usize = 6;

macro_rules! write_self_indent {
    ($self: ident, $content: expr) => {
        write!($self.f, "{: <1$}", $content, $self.indent)
    };
}

macro_rules! write_self {
    ($self: ident $(, $content: expr)*) => {
        write!($self.f $(, $content)*)
    };
}

impl AstMutVisitor<'_, (), Error> for PrettyPrinterVisitor<'_, '_> {
    fn default_t(_: super::DefaultCause) -> miette::Result<(), Error> {
        Ok(())
    }

    fn visit_statement(
        &mut self,
        statement: &crate::ast::nodes::statements::Statement,
    ) -> miette::Result<(), Error> {
        self.default_statement(statement)?;
        write_self_indent!(self, "\n")
    }

    fn visit_expression(
        &mut self,
        expression: &crate::ast::nodes::expressions::Expression,
    ) -> miette::Result<(), Error> {
        let depth = expression_max_depth(expression).unwrap_or(0);
        self.indent += IDENT;
        if depth > 1 {
            write_self!(self, "(")?;
        }
        if depth > 2 {
            write_self_indent!(self, "\n")?;
        }
        self.default_expression(expression)?;
        self.indent -= IDENT;
        if depth > 2 {
            write_self_indent!(self, "\n")?;
        }
        if depth > 1 {
            write_self!(self, ")")?;
        }
        Ok(())
    }

    fn visit_deprecated(
        &mut self,
        deprecated: &crate::ast::nodes::deprecated::Deprecated,
    ) -> miette::Result<(), Error> {
        self.default_deprecated(deprecated)?;
        write_self!(self, "DEPRECATED [{}]", deprecated.message)
    }

    fn visit_function_call(
        &mut self,
        function_call: &crate::ast::nodes::calls::functions::FunctionCall,
    ) -> miette::Result<(), Error> {
        // Isolate to release the lock on the interner
        {
            let interner = get_interner_typed()?;
            let name = interner
                .resolve(function_call.name.into())
                .unwrap_or("ERROR");

            write_self!(self, "{}(", name)?;
        }
        self.default_function_call(function_call)?;
        write_self!(self, ")")
    }

    fn visit_number(
        &mut self,
        number: &crate::ast::nodes::numbers::Number,
    ) -> miette::Result<(), Error> {
        let interner = get_interner_typed()?;
        let name = interner.resolve(number.content.into()).unwrap_or("ERROR");
        write_self!(self, "{}", name)
    }

    fn visit_binop(
        &mut self,
        binop: &crate::ast::nodes::binop::Binop,
    ) -> miette::Result<(), Error> {
        self.visit_expression(&binop.left)?;
        write_self!(self, " {} ", binop.binop)?;
        self.visit_expression(&binop.right)
    }

    fn visit_variable_declaration(
        &mut self,
        variable_declaration: &crate::ast::nodes::declarations::variable::VariableDeclaration,
    ) -> miette::Result<(), Error> {
        {
            let interner = get_interner_typed()?;
            let name_type = interner
                .resolve(variable_declaration.allocated_type.symbol)
                .unwrap_or("ERROR");
            let name = interner
                .resolve(variable_declaration.name.symbol)
                .unwrap_or("ERROR");
            write_self!(self, ". {name_type} {name} (")?;
        }

        self.default_variable_declaration(variable_declaration)?;
        write_self!(self, ")")
    }

    fn visit_variable_usage(
        &mut self,
        variable_usage: &crate::ast::nodes::calls::variable::VariableUsage,
    ) -> miette::Result<(), Error> {
        let interner = get_interner_typed()?;
        let name = interner
            .resolve(variable_usage.name.symbol)
            .unwrap_or("ERROR");
        write_self!(self, "{name}")
    }
}
