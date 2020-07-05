SET log_statement = 'all';
SET TIME ZONE 'Japan';

DROP TABLE IF EXISTS users;

CREATE TABLE users (
  id BIGSERIAL PRIMARY KEY,
  name VARCHAR(255) NOT NULL,
  title VARCHAR(255),
  age INTEGER NOT NULL,
  active BOOLEAN NOT NULL,
  created_at TIMESTAMP WITH TIME ZONE NOT NULL
);

INSERT INTO users(name, title, age, active, created_at) VALUES
 ('ユーザ1', '旅人', 20, TRUE, CURRENT_TIMESTAMP)
,('ユーザ2', NULL, 21, FALSE, CURRENT_TIMESTAMP)
,('ユーザ3', 'もののけ', 22, TRUE, CURRENT_TIMESTAMP)
;
