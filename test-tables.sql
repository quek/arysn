SET log_statement = 'all';
SET TIME ZONE 'Japan';

DROP TABLE IF EXISTS users;

CREATE TABLE users (
  id BIGSERIAL PRIMARY KEY,
  name VARCHAR(255) NOT NULL
);

INSERT INTO users(name) VALUES
 ('ユーザ1')
,('ユーザ2')
,('ユーザ3')
;
