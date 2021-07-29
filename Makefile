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

test:
	docker-compose exec dev cargo test --features "gis" -- --nocapture

test-no-gis:
	docker-compose exec dev cargo test -- --nocapture

test-tokio-02:
	docker-compose exec dev cargo test --no-default-features \
	  --features "with-tokio-0_2" --features "gis-tokio-0_2" -- --nocapture
