SET log_statement = 'all';
SET TIME ZONE 'Japan';

DROP TABLE IF EXISTS contributions;
DROP TABLE IF EXISTS projects;
DROP TABLE IF EXISTS screens;
DROP TABLE IF EXISTS roles;
DROP TYPE IF EXISTS role_type;
DROP TABLE IF EXISTS users;

CREATE TABLE users (
  id BIGSERIAL PRIMARY KEY,
  name VARCHAR(255) NOT NULL,
  title VARCHAR(255),
  age INTEGER NOT NULL,
  active BOOLEAN NOT NULL,
  created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO users(name, title, age, active, created_at) VALUES
 ('ユーザ1', '旅人', 20, TRUE, CURRENT_TIMESTAMP)
,('ユーザ2', NULL, 21, FALSE, CURRENT_TIMESTAMP)
,('ユーザ3', 'もののけ', 22, TRUE, CURRENT_TIMESTAMP)
;

CREATE TYPE role_type AS ENUM ('admin', 'user');

CREATE TABLE roles (
  id BIGSERIAL PRIMARY KEY,
  user_id BIGINT NOT NULL REFERENCES users ON DELETE CASCADE,
  role_type role_type NOT NULL
);

INSERT INTO roles(user_id, role_type) VALUES
 (1, 'admin')
,(1, 'user')
,(2, 'user')
;

CREATE TABLE screens (
  id BIGSERIAL PRIMARY KEY,
  role_id BIGINT NOT NULL REFERENCES roles ON DELETE CASCADE,
  name VARCHAR(255) NOT NULL
);

INSERT INTO screens(role_id, name) VALUES
 (1, 'ねこ')
,(1, 'かも')
,(2, 'さくらえび')
,(3, 'のり')
;

CREATE TABLE projects (
  id BIGSERIAL PRIMARY KEY,
  name VARCHAR(255) NOT NULL,
  parent_project_id BIGINT REFERENCES projects ON DELETE RESTRICT,
  create_user_id BIGINT NOT NULL REFERENCES users ON DELETE RESTRICT,
  update_user_id BIGINT NOT NULL REFERENCES users ON DELETE RESTRICT
);

INSERT INTO projects (name, parent_project_id, create_user_id, update_user_id) VALUES
 ('ねこの手企画(1)', null, 1, 1)
,('ねこしっぽ組(2)', 1, 2, 2)
,('ねこみみ係(3)', 2, 1, 2)
;

CREATE TABLE contributions (
  id BIGSERIAL PRIMARY KEY,
  project_id BIGINT NOT NULL REFERENCES projects ON DELETE CASCADE,
  user_id BIGINT NOT NULL REFERENCES users ON DELETE CASCADE
);

INSERT INTO contributions (project_id, user_id) VALUES
 (1, 1)
,(2, 1)
,(3, 1)
,(1, 2)
;
