LAB ?= 
LAB_DIR := labs/$(LAB)
CONFIG ?= config.toml

COMPOSE = docker compose

up: down
	$(COMPOSE) up -d

down:
	$(COMPOSE) down -v

.PHONY: guard-config
guard-config:
	@if [ ! -f "$(CONFIG)" ]; then \
	  echo "ERROR: Root config '$(CONFIG)' not found."; \
	  echo "Create it (e.g., copy config.example.toml) or pass CONFIG=<path>."; \
	  exit 1; \
	fi

.PHONY: sub
sub: guard-config
	cargo run --manifest-path shared/Cargo.toml --bin nats-sub -- \
	  --config $(CONFIG) --lab-config $(LAB_DIR)/config.toml $(ARGS)

.PHONY: pub
pub: guard-config
	@if [ -z "$(MSG)" ]; then echo "Provide MSG=\"your message\""; exit 1; fi
	cargo run --manifest-path shared/Cargo.toml --bin nats-pub -- \
	  --config $(CONFIG) --lab-config $(LAB_DIR)/config.toml --msg "$(MSG)" $(ARGS)
