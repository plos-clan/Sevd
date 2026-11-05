use crate::compiler::com_error::ParserError;
use crate::compiler::ir::AstNode;
use crate::compiler::parser::Parser;
use crate::compiler::parser::expr::{ExprTerminator, ExprType, get_of_end_or_block_end_expr};

pub fn return_parser(parser: &mut Parser) -> Result<AstNode, ParserError> {
    let last = parser.get_token()?;
    let expr = get_of_end_or_block_end_expr(parser, &last, ExprType::Init)?;
    if let ExprTerminator::BlockEnd = expr.1 {
        return Err(ParserError::Expected(last, ';'));
    }
    Ok(AstNode::Return(Box::new(expr.0)))
}
