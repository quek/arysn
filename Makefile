export UID=$(shell id -u)
export GID=$(shell id -g)

all:
	mkdir -p tmp/.cargo
	docker-compose up --build
