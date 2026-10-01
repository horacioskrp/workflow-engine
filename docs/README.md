# Documentation

Documentation d'ingénierie du moteur de workflow (Rust).

| Document | Contenu |
|---|---|
| [architecture.md](architecture.md) | **Constitution** du projet : vision, couches, les 15 crates (gros briques), graphe de dépendances, modèle d'exécution runtime, décisions structurantes, points durs. |
| [roadmap.md](roadmap.md) | **Toutes les étapes d'implémentation**, phase par phase (0 → 5) : objectifs, crates touchées, étapes concrètes, livrables (Definition of Done), dépendances externes, risques. |

## État actuel

**Phase 0 — scaffold livré.** Le workspace Cargo (16 crates), le câblage, les lints,
le contrat gRPC (`proto/gateway.proto`) et un daemon `broker` qui démarre sont en
place. Les crates sont des stubs documentés ; aucune logique métier encore.

Le projet compile et s'exécute via Docker (voir le [README racine](../README.md)).

## Fidélité au projet source

L'architecture et les phases ont été vérifiées contre les `pom.xml` réels du projet
Java d'origine. Correspondance des modules → crates :

| Module(s) source | Crate(s) |
|---|---|
| `protocol`, `protocol-impl`, `msgpack-*`, `bpmn-model`, `protocol-jackson` | `model` |
| `expression-language` | `feel` |
| `journal`, `logstreams`, `dispatcher` | `journal` |
| `zb-db` (`db`), `snapshot` | `state` |
| `atomix` | `cluster` + `transport` |
| `engine` (`workflow-engine`) | `engine` |
| `gateway`, `gateway-protocol`, `gateway-protocol-impl` | `gateway` + `proto` |
| `exporters`, `exporter-api` | `exporter` |
| `broker`, `dist` | `broker` |
| `clients` (java/go/oauth2) | `client` + `cli` |
| `util`, `test-util`, `protocol-test-util` | `common` + `testkit` |
| `build-tools` | `xtask` |
| `monitor` (monitoring) | transverse : observabilité (`tracing`/metrics), pas une brique |
| `bom`, `parent`, `qa`, `samples`, `benchmarks`, `docker`, `docs` | infra / tests / non applicable |

Deux dépendances alignées sur le source après vérification :
- `engine` → `journal` : le stream processor **lit** le log committé (le source :
  `engine` dépend de `logstreams`).
- `state` ne dépend **pas** de `model` : c'est un store KV générique (le source :
  `zb-db` est générique). Les vues d'état typées vivent dans `engine::state`.

## Rappels de cadrage

- **Compatibilité visée : API-only.** On reproduit le contrat client (gRPC) mais le
  format interne (log, snapshots, protocole réseau) est le nôtre — pas de migration
  en place depuis un système existant. C'est le choix le plus réaliste.
- **Conventions** : Pragmatic Rust Guidelines (ids `M-*` cités dans les docs et le
  code). Édition 2024, `unsafe` interdit par défaut, erreurs canoniques en structs,
  `anyhow` dans les binaires.
- Les trois **points durs** connus : le consensus (Raft), le langage d'expression
  (FEEL), et le **déterminisme** du stream processor. Détaillés dans
  [architecture.md](architecture.md#points-durs).
