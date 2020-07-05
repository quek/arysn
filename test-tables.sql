SET log_statement = 'all';
SET TIME ZONE 'Japan';

DROP TABLE IF EXISTS users;

CREATE TABLE users (
  id BIGSERIAL PRIMARY KEY,
  name VARCHAR(255) NOT NULL,
  title VARCHAR(255),
  active BOOLEAN NOT NULL,
  created_at TIMESTAMP WITH TIME ZONE NOT NULL
);

INSERT INTO users(name, title, active, created_at) VALUES
 ('ユーザ1', '旅人', TRUE, CURRENT_TIMESTAMP)
,('ユーザ2', NULL, FALSE, CURRENT_TIMESTAMP)
,('ユーザ3', 'もののけ', TRUE, CURRENT_TIMESTAMP)
;
