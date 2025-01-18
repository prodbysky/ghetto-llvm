Program: [Statement];
Statement:
    Let
    Exit

Let: let `name`: `type` = [Expression]
Exit: exit([Expression])


Expression:
    BinaryExpression
    Number

BinaryExpression:
    Left: [Expression]
    Operator: BinaryOperator
    Right: [Expression]

Number: 
    [0-9]*

BinaryOperator:
    `+`
    `-`
    `*`

