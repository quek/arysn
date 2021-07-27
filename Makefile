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

test-no-gis:
	docker exec --workdir /app/arysn-test arysn_dev_1 cargo test -- --nocapture

test-tokio-02:
	docker-compose exec dev cargo test --features "with-tokio-0_2" --no-default-features -- --nocapture
