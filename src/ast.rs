
#[derive(Debug, Clone)]
pub enum Expr {
    Number(f64),
    String(String),
    Boolean(bool),
    Variable(String),

    Array(Vec<Expr>),

    Index {
        array: Box<Expr>,
        index: Box<Expr>,
    },

    Binary {
        left: Box<Expr>,
        operator: String,
        right: Box<Expr>,
    },

    Unary {
        operator: String,
        expression: Box<Expr>,
    },

    Call {
        name: String,
        arguments: Vec<Expr>,
    },
}

#[derive(Debug, Clone)]
pub enum Statement {
    Let(String, Expr),
    Set(String, Expr),

    SetIndex {
        name: String,
        index: Expr,
        value: Expr,
    },

    Append {
        value: Expr,
        array: String,
    },

    Say(Expr),
    Expression(Expr),

    Function {
        name: String,
        parameters: Vec<String>,
        body: Vec<Statement>,
    },

    If {
        condition: Expr,
        then_branch: Vec<Statement>,
        else_branch: Vec<Statement>,
    },

    Repeat {
        count: Expr,
        body: Vec<Statement>,
    },
}

#[derive(Debug, Clone)]
pub struct Program {
    pub statements: Vec<Statement>,
}
