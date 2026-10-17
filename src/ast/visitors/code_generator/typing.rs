use inkwell::{
    types::{AnyTypeEnum, BasicMetadataTypeEnum, BasicTypeEnum, FunctionType, IntType},
    values::{BasicValueEnum, FunctionValue, IntValue, PointerValue},
};
use miette::{Error, Result, miette};

#[derive(Debug, Clone)]
pub struct SimpleValue<'a> {
    pub value: BasicValueEnum<'a>,
    pub typed: BasicTypeEnum<'a>,
}

pub struct SimpleIntValue<'a> {
    pub value: IntValue<'a>,
    pub typed: IntType<'a>,
}

pub struct SimpleFunctionValue<'a> {
    pub value: FunctionValue<'a>,
    pub return_type: Option<BasicTypeEnum<'a>>,
}

impl<'a> TryFrom<SimpleValue<'a>> for SimpleIntValue<'a> {
    type Error = Error;

    fn try_from(value: SimpleValue<'a>) -> std::prelude::v1::Result<Self, Self::Error> {
        let new_value = value
            .value
            .try_into()
            .map_err(|_| miette!("Cannot convert value to int"))?;
        let new_type = value
            .typed
            .try_into()
            .map_err(|_| miette!("Cannot convert type to int"))?;

        Ok(SimpleIntValue {
            value: new_value,
            typed: new_type,
        })
    }
}

impl<'a> From<SimpleIntValue<'a>> for SimpleValue<'a> {
    fn from(value: SimpleIntValue<'a>) -> Self {
        let new_value = value.value.into();
        let new_type = value.typed.into();

        SimpleValue {
            value: new_value,
            typed: new_type,
        }
    }
}

#[derive(Debug)]
pub struct SimplePointerValue<'a> {
    pub value: PointerValue<'a>,
    pub pointed_type: BasicTypeEnum<'a>,
}

#[derive(Debug)]
pub enum Variable<'a> {
    SimpleValue(SimpleValue<'a>),
    PointerValue(SimplePointerValue<'a>),
}

pub type TypeType<'a> = BasicTypeEnum<'a>;
pub type ValueType<'a> = BasicValueEnum<'a>;
pub type GeneralValue<'a> = Option<SimpleValue<'a>>;

pub(super) fn to_fn_type<'ctx>(
    return_type: AnyTypeEnum<'ctx>,
    parameters_types: &[BasicMetadataTypeEnum<'ctx>],
    is_var_args: bool,
) -> Result<FunctionType<'ctx>> {
    Ok(match return_type {
        AnyTypeEnum::IntType(return_type) => return_type.fn_type(parameters_types, is_var_args),
        // Same code...
        // Just the type is changed
        AnyTypeEnum::VoidType(return_type) => return_type.fn_type(parameters_types, is_var_args),
        _ => Err(miette!("Type not supported for return type"))?,
    })
}
