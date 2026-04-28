.PHONY: help setup dev dev-python prod down logs ps proto clean shell-rust shell-python

COMPOSE_PROD = docker compose --env-file $(CURDIR)/.env -f .docker/docker-compose.yml
COMPOSE_DEV  = docker compose --env-file $(CURDIR)/.env -f .docker/docker-compose.yml -f .docker/docker-compose.dev.yml

## help: Print this help
help:
	@grep -E '^## ' Makefile | sed 's/## //'

## setup: Copia .env.template → .env si no existe
setup:
	@test -f .env || (cp .env.template .env && echo ".env creado — completá los valores antes de continuar.")

## dev-detached: Levanta en background (para dejar corriendo y salir)
dev-detached: setup
	$(COMPOSE_DEV) up --build -d

dev: setup
	$(COMPOSE_DEV) up --build --no-deps horbyta_app -d

up: setup
	$(COMPOSE_DEV) up --no-deps horbyta_app -d

## prod: Levanta en producción (detached)
prod: setup
	$(COMPOSE_PROD) up --build -d

## down: Para todos los servicios
down:
	$(COMPOSE_DEV) down

## logs: Seguir logs en tiempo real
logs:
	$(COMPOSE_DEV) logs -f

## logs-rust: Solo logs de Rust
logs:
	$(COMPOSE_DEV) logs -f horbyta_app

## ps: Ver estado y health de los contenedores
ps:
	$(COMPOSE_DEV) ps

## shell-rust: Entrar al contenedor Rust
shell:
	$(COMPOSE_DEV) exec horbyta_app bash

## clean: Elimina contenedores, imágenes locales y volúmenes del proyecto
clean:
	$(COMPOSE_DEV) down --volumes --rmi local
