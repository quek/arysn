# Changelog

## 0.5.2

- Add join_select.

## 0.5.1

- Fix, r#where and or

## 0.5.0

- tokio 0.2 is not supported.
- update dependencies
    - deadpool to 0.9
    - deadpool-postgres to 0.10

## 0.4.0

- Derive can be specified.
- Add group_by_literal.

## 0.3.7

- Derive PartialEq.

## 0.3.6

- Add `literal_condition`.

## 0.3.5

- Add LIKE operator.

## 0.3.4

- Remove duplicate id in preload of belongs to.

## 0.3.3

- Fix belongs to struct_name: "FooStatus"

## 0.3.2

- Add feature `with-tokio-0_2`
- Reduced the size of the generated code

## 0.3.1

- fix han one nullable foreign key

## 0.3.0

- update dependencies
    - deadpool to 0.8
    - deadpool-postgres to 0.9
    - tokio to 1.9
    - tokio-postgres to 0.7
    - env_logger to 0.9
    - bytes to 1.0
