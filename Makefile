export UID=$(shell id -u)
export GID=$(shell id -g)

all:
	mkdir -p tmp/.cargo
	docker-compose up --build


create-test-tables:
	cat test-tables.sql | docker exec -i arysn_db_1 psql -U user1 arysn_development

psql:
	docker exec -it arysn_db_1 psql -U user1 arysn_development
