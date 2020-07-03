SET log_statement = 'all';
SET TIME ZONE 'Japan';

DROP TABLE IF EXISTS users;

CREATE TABLE users (
  id BIGSERIAL PRIMARY KEY,
  name VARCHAR(255) NOT NULL,
  title VARCHAR(255)
);

INSERT INTO users(name, title) VALUES
 ('ユーザ1', '旅人')
,('ユーザ2', null)
,('ユーザ3', 'もののけ')
;
