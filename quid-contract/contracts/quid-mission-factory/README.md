# Quid Mission Factory

Registry of curated mission defaults that launches fully funded missions in a
configured `quid-store` with one call.

## Safety model

- A single admin registers templates; templates are immutable.
- Every template pins its target store, reward token and amount, participant
  cap, optional asset gate, and mission metadata.
- Empty metadata, zero/negative rewards, zero participant caps, inconsistent
  asset gates, and overflowing escrow totals are rejected.
- The owner must authorize `create_from_template` and pays the store escrow
  (plus any protocol fee configured on that store).
- The registry is capped at 100 entries, keeping `list_templates` bounded.

## API

- `initialize(admin)` — bootstrap the registry once.
- `register_template(config) -> template_id` — admin-only registration.
- `create_from_template(template_id, owner) -> mission_id` — create in the
  template's store using its exact defaults.
- `get_template(template_id)` / `list_templates()` / `template_count()`.
- `get_admin()`.

## Testnet setup

Deploy `quid-store` first, then build and deploy the factory:

```bash
stellar contract build
stellar contract deploy \
  --wasm target/wasm32v1-none/release/quid_mission_factory.wasm \
  --source alice \
  --network testnet

stellar contract invoke \
  --id <MISSION_FACTORY_CONTRACT_ID> \
  --source alice \
  --network testnet \
  -- initialize --admin alice
```

Use `stellar contract invoke --help` (and the generated contract spec) to pass
the `TemplateConfig` map to `register_template`. Before creation, the owner
must hold and authorize enough reward tokens for
`reward_amount * max_participants`, plus the store's configured protocol fee.

## Tests

```bash
cargo test -p quid-mission-factory
```

The integration test registers a real `quid-store`, creates a mission through
the factory, verifies its stored fields, and verifies the escrow balance.
