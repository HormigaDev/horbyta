.PHONY: help setup dev dev-python prod down logs ps proto clean shell-rust shell-python

COMPOSE_PROD = docker compose -f .docker/docker-compose.yml
COMPOSE_DEV  = docker compose -f .docker/docker-compose.yml -f .docker/docker-compose.dev.yml
COMPOSE_ENV  = --env-file $(CURDIR)/.env

## help: Print this help
help:
	@grep -E '^## ' Makefile | sed 's/## //'

## setup: Copia .env.template → .env si no existe
setup:
	@test -f .env || (cp .env.template .env && echo ".env creado — completá los valores antes de continuar.")

## dev: Levanta todos los servicios en modo desarrollo
dev: setup
	$(COMPOSE_ENV) $(COMPOSE_DEV) up --build

## dev-detached: Levanta en background (para dejar corriendo y salir)
dev-detached: setup
	$(COMPOSE_ENV) $(COMPOSE_DEV) up --build -d

dev-python: setup
	$(COMPOSE_ENV) $(COMPOSE_DEV) up --build horbyta_ai -d

dev-rust: setup
	$(COMPOSE_ENV) $(COMPOSE_DEV) up --build horbyta_app -d

## prod: Levanta en producción (detached)
prod: setup
	$(COMPOSE_ENV) $(COMPOSE_PROD) up --build -d

## down: Para todos los servicios
down:
	$(COMPOSE_ENV) $(COMPOSE_DEV) down

## logs: Seguir logs en tiempo real
logs:
	$(COMPOSE_ENV) $(COMPOSE_DEV) logs -f

## logs-rust: Solo logs de Rust
logs-rust:
	$(COMPOSE_ENV) $(COMPOSE_DEV) logs -f horbyta_app

## logs-python: Solo logs del servicio IA
logs-python:
	$(COMPOSE_ENV) $(COMPOSE_DEV) logs -f horbyta_ai

## ps: Ver estado y health de los contenedores
ps:
	$(COMPOSE_ENV) $(COMPOSE_DEV) ps

## shell-rust: Entrar al contenedor Rust
shell-rust:
	$(COMPOSE_ENV) $(COMPOSE_DEV) exec horbyta_app bash

## shell-python: Entrar al contenedor Python
shell-python:
	$(COMPOSE_ENV) $(COMPOSE_DEV) exec horbyta_ai bash

## clean: Elimina contenedores, imágenes locales y volúmenes del proyecto
clean:
	$(COMPOSE_ENV) $(COMPOSE_DEV) down --volumes --rmi local