build:
	docker compose build
	docker compose run app sh -c "cargo build"
up:
	docker compose up -d
down:
	docker compose down --rmi all --volumes --remove-orphans
init:
	docker compose exec app bash