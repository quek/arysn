export UID=$(shell id -u)
export GID=$(shell id -g)

all:
	mkdir -p tmp/.cargo
	docker-compose up --build


create-test-tables:
	cat test-tables.sql | docker exec -i arysn_db_1 psql -U user1 arysn_development

recreate-test-tables:
	docker-compose down
	docker volume rm arysn_postgresql_data
	docker-compose up --build

psql:
	docker exec -it arysn_db_1 psql -U user1 arysn_development

publish:
	cargo publish --manifest-path arysn/Cargo.toml

test-tokio-1_x:
	docker-compose exec dev cargo test -p arysn-test \
	  --no-default-features \
	  --features "with-tokio-1_x" -- --nocapture

test-tokio-1_x-gis:
	docker-compose exec dev cargo test -p arysn-test \
	  --no-default-features \
	  --features "with-tokio-1_x-gis" -- --nocapture

test-tokio-0_2:
	docker-compose exec dev cargo test -p arysn-test \
	  --no-default-features \
	  --features "with-tokio-0_2" \
	  -- --nocapture

test-tokio-0_2-gis:
	docker-compose exec dev cargo test -p arysn-test \
	  --no-default-features \
	  --features "with-tokio-0_2-gis" \
	  -- --nocapture

test-all: test-tokio-1_x test-tokio-1_x-gis test-tokio-0_2 test-tokio-0_2-gis
