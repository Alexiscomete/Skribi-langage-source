use std::collections::HashMap;
use std::fs::create_dir_all;
use std::path::Path;

use inkwell::AddressSpace;
use inkwell::basic_block::BasicBlock;
use inkwell::context::Context as InkContext;
use inkwell::module::Linkage;
use inkwell::types::{AnyType, AnyTypeEnum, BasicMetadataTypeEnum, BasicTypeEnum, FunctionType};
use inkwell::values::{
    BasicMetadataValueEnum, BasicValueEnum, FunctionValue, IntValue, PointerValue,
};
use inkwell::{builder::Builder, module::Module};
use log::{debug, trace};
use miette::{Context, IntoDiagnostic, Result, miette};

use crate::ast::nodes::binop::BinopEnum;
use crate::ast::nodes::into_str;
use crate::ast::{nodes::FileTreeRoot, visitors::AstMutVisitor};
use crate::interner::get_interner;

type ExpectedTypeType<'a> = BasicTypeEnum<'a>;
type TypeType<'a> = Option<ExpectedTypeType<'a>>;
type ValueType<'a> = BasicValueEnum<'a>;
type Ret<'a> = Option<(BasicValueEnum<'a>, TypeType<'a>)>;

pub struct CodeGenerator<'ctx> {
    context: &'ctx InkContext,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    main_empty: bool,
    var_values: HashMap<usize, (PointerValue<'ctx>, ExpectedTypeType<'ctx>)>,
}

impl<'ctx> CodeGenerator<'ctx> {
    fn goin(&self, block: BasicBlock) {
        self.builder.position_at_end(block);
    }

    fn to_fn_type(
        return_type: AnyTypeEnum<'ctx>,
        parameters_types: &[BasicMetadataTypeEnum<'ctx>],
        is_var_args: bool,
    ) -> Result<FunctionType<'ctx>> {
        Ok(match return_type {
            AnyTypeEnum::IntType(return_type) => return_type.fn_type(parameters_types, is_var_args),
            // Same code...
            // Just the type is changed
            AnyTypeEnum::VoidType(return_type) => {
                return_type.fn_type(parameters_types, is_var_args)
            }
            _ => Err(miette!("Type not supported for return type"))?,
        })
    }

    fn to_int_math_value(value: ValueType<'ctx>) -> Result<IntValue<'ctx>> {
        Ok(match value {
            ValueType::IntValue(int_value) => int_value,
            _ => Err(miette!("Type not supported int type"))?,
        })
    }

    fn import_function(
        &self,
        name: &str,
        return_type: TypeType<'ctx>,
        parameters_types: &[BasicMetadataTypeEnum<'ctx>],
        is_var_args: bool,
        linkage: Option<Linkage>,
    ) -> Result<FunctionValue<'ctx>> {
        let main_function_type = Self::to_fn_type(
            return_type.map_or_else(|| self.context.void_type().into(), |v| v.as_any_type_enum()),
            parameters_types,
            is_var_args,
        )?;
        let main_function = self.module.add_function(name, main_function_type, linkage);
        Ok(main_function)
    }

    fn get_or_import(
        &self,
        name: &str,
        return_type: TypeType<'ctx>,
        parameters_types: &[BasicMetadataTypeEnum<'ctx>],
        is_var_args: bool,
        linkage: Option<Linkage>,
    ) -> Result<(FunctionValue<'ctx>, TypeType<'ctx>)> {
        Ok(if let Some(func) = self.module.get_function(name) {
            (func, return_type)
        } else {
            let func =
                self.import_function(name, return_type, parameters_types, is_var_args, linkage)?;

            trace!("Function {} declared", name);

            (func, return_type)
        })
    }

    /// This function is made to be a simplified way to create fonctions
    /// This is not a way to import fonctions
    fn create_function(
        &self,
        name: &str,
        return_type: TypeType<'ctx>,
        parameters_types: &[BasicMetadataTypeEnum<'ctx>],
        is_var_args: bool,
        linkage: Option<Linkage>,
    ) -> Result<BasicBlock<'_>> {
        // TODO: add void type
        let main_function =
            self.import_function(name, return_type, parameters_types, is_var_args, linkage)?;
        let main_block = self.context.append_basic_block(main_function, name);
        Ok(main_block)
    }

    fn create_base(&self) -> Result<()> {
        // Create main function
        // TODO: add arguments
        let ret_type = self.context.i32_type();
        let function = self.create_function("main", Some(ret_type.into()), &[], false, None)?;
        self.goin(function);
        Ok(())
    }

    fn save(&self, name: &str, folder: &str) -> Result<()> {
        let path = Path::new(folder).join(name).with_added_extension("ll");
        let parent = path.as_path().parent().context("No parent folder")?;
        create_dir_all(parent)
            .into_diagnostic()
            .context(format!("Cannot create folders for `{}`", name))?;
        let path_str = path.to_str().context("Invalid path format")?.to_owned();
        self.module
            .print_to_file(path)
            .into_diagnostic()
            .context(format!(
                "Failed to save program in HIR format file {}",
                path_str
            ))?;
        Ok(())
    }

    fn build_exit_call(&self, code: ValueType<'ctx>) -> Result<()> {
        let argument_type = self.context.i32_type();

        let exit_function =
            self.get_or_import("exit", None, &[argument_type.into()], false, None)?;

        // We might want to simplify this later
        // Not enough data for now
        self.builder
            .build_call(exit_function.0, &[code.into()], "call_exit")
            .into_diagnostic()
            .context("While creating call to exit")?;
        self.builder
            .build_unreachable()
            .into_diagnostic()
            .context("While creating unreachable end of branch")?;

        Ok(())
    }

    fn build_print_fct(&self) -> Result<(FunctionValue<'ctx>, TypeType<'ctx>)> {
        let argument_type = self.context.ptr_type(AddressSpace::default());
        let return_type = self.context.i32_type();

        self.get_or_import(
            "printf",
            Some(return_type.into()),
            &[argument_type.into()],
            true,
            None,
        )
    }

    fn create_format_str(
        &self,
        value: Option<ValueType<'ctx>>,
        name: &str,
    ) -> Result<PointerValue<'ctx>> {
        Ok(self
            .builder
            .build_global_string_ptr(if value.is_some() { "%i\n" } else { "\n" }, name)
            .into_diagnostic()?
            .as_pointer_value())
    }

    fn build_print_call(&self, value: Option<ValueType<'ctx>>) -> Result<Ret<'ctx>> {
        let print = self.build_print_fct()?;
        let format = self.create_format_str(value, "call_printf")?;
        let args: &[BasicMetadataValueEnum<'ctx>] = if let Some(value) = value {
            &[format.into(), value.into()]
        } else {
            &[format.into()]
        };

        Ok(Some((
            self.builder
                .build_call(print.0, args, "call_printf")
                .into_diagnostic()
                .context("While creating call to printf")?
                .try_as_basic_value()
                .expect_basic("Printf always returns")
                .into(),
            print.1,
        )))
    }

    pub fn compile(root: &FileTreeRoot, name: &str, folder: &str) -> Result<()> {
        let context = InkContext::create();
        let module = context.create_module(name);
        let builder = context.create_builder();

        let mut compiler = CodeGenerator {
            context: &context,
            module,
            builder,
            main_empty: true,
            var_values: HashMap::new(),
        };
        compiler.create_base()?;
        compiler.visit_file_tree_root(root)?;

        if compiler.main_empty {
            let arg = compiler.context.i32_type().const_int(0, false);
            compiler.build_exit_call(arg.into())?;
        }

        compiler.save(name, folder)?;

        Ok(())
    }
}

impl<'ctx> AstMutVisitor<'_, Ret<'ctx>> for CodeGenerator<'ctx> {
    fn default_t(_: super::DefaultCause) -> miette::Result<Ret<'static>, miette::Error> {
        Ok(None)
    }

    fn visit_function_call(
        &mut self,
        function_call: &crate::ast::nodes::calls::functions::FunctionCall,
    ) -> Result<Ret<'ctx>, miette::Error> {
        trace!("Compiling a function call");

        // Step 1, compile the arguments
        // TODO: support multiple arguments

        let args = self.default_function_call(function_call)?;

        // Step 2, call
        let interner = get_interner()?;
        let name = into_str(&interner, function_call.name);
        match name {
            "exit" => {
                trace!("Found an exit call");
                let arg = args.map_or_else(
                    || self.context.i32_type().const_int(42, false).into(),
                    |v| v.0,
                );
                self.build_exit_call(arg)?;
                self.main_empty = false;
                trace!("Function called");

                Ok(None)
            }
            "println" => self.build_print_call(args.map(|x| x.0)),
            _ => todo!("Cannot compile other functions for now"),
        }
    }

    fn visit_number(
        &mut self,
        number: &crate::ast::nodes::numbers::Number,
    ) -> Result<Ret<'ctx>, miette::Error> {
        trace!("Compiling a number");

        let interner = get_interner()?;
        let name = into_str(&interner, number.content);
        let value = name.parse().into_diagnostic()?;

        debug!("Found number {}", value);

        let res = self.context.i32_type().const_int(value, false);
        Ok(Some((res.into(), Some(self.context.i32_type().into()))))
    }

    fn visit_binop(
        &mut self,
        binop: &crate::ast::nodes::binop::Binop,
    ) -> Result<Ret<'ctx>, miette::Error> {
        trace!("Compiling a binop");

        let base_left = self
            .visit_expression(&binop.left)?
            .map_or_else(|| Err(miette!("No valid left expression")), Ok)?;
        let base_right = self
            .visit_expression(&binop.right)?
            .map_or_else(|| Err(miette!("No valid right expression")), Ok)?;

        let left = Self::to_int_math_value(base_left.0)?;
        let right = Self::to_int_math_value(base_right.0)?;

        let result = match binop.binop {
            BinopEnum::Add => self.builder.build_int_add(left, right, "add"),
            BinopEnum::Substract => self.builder.build_int_sub(left, right, "sub"),
            BinopEnum::Multiply => self.builder.build_int_mul(left, right, "mul"),
            BinopEnum::Divide => self.builder.build_int_signed_div(left, right, "div"),
        }
        .into_diagnostic()?;

        debug!("Types are {:?} and {:?}", base_left.1, base_right.1);

        Ok(Some((result.into(), base_left.1)))
    }

    fn visit_variable_declaration(
        &mut self,
        variable_declaration: &crate::ast::nodes::declarations::variable::VariableDeclarationRef,
    ) -> Result<Ret<'ctx>, miette::Error> {
        trace!("Compiling a variable declaration");

        let content = self
            .default_variable_declaration(variable_declaration)?
            .wrap_err("No valid content")?;

        debug!("Declaring {:?}", content);

        let content_type = content.1.wrap_err("No valid type")?;

        let allocated = self
            .builder
            .build_alloca(content_type, "var")
            .into_diagnostic()?;

        self.builder.build_store(allocated, content.0).into_diagnostic()?;

        self.var_values
            .insert(variable_declaration.read(|v| v.id), (allocated, content_type));

        Ok(Some(content))
    }

    fn visit_variable_usage(
        &mut self,
        variable_usage: &crate::ast::nodes::calls::variable::VariableUsage,
    ) -> Result<Ret<'ctx>, miette::Error> {
        trace!("Compiling a variable usage");

        let stored = self
            .var_values
            .get(
                &variable_usage
                    .declaration
                    .clone()
                    .wrap_err("Undeclared declared variable")?
                    .read(|v| v.id),
            )
            .wrap_err("Declared variable was removed from AST")?
            .clone();

        debug!("Stored is {:?}", stored);

        Ok(Some((
            self.builder
                .build_load(stored.1, stored.0, "var")
                .into_diagnostic()?,
            Some(stored.1),
        )))
    }
}
